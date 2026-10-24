#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#import <objc/message.h>
#import <objc/runtime.h>
#import <fcntl.h>
#import <unistd.h>
#import <stdlib.h>
#import <string.h>
#import <errno.h>

static const char *kExpectedSource = "com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese";
static int traceFD = -1;
static NSString *preparePath;
static NSTimer *prepareTimer;
static BOOL modeReady = NO;
static IMP originalSendEvent = NULL;
static void installSendEventObserver(void);
static NSString *lastPreparePendingState;
static BOOL sendEventObserverInstalled = NO;
static id observedTextClient = nil;
static IMP originalSetMarkedText = NULL;
static IMP originalInsertText = NULL;
static IMP originalUnmarkText = NULL;
static BOOL textClientObserverInstalled = NO;

static void installTextClientObserver(NSView *client);

static void observerFatal(NSString *message) {
    fprintf(stderr, "macOS native observer: %s\n", message.UTF8String ?: "unknown error");
    fflush(stderr);
    _exit(1);
}

static void writeAll(const char *bytes, size_t length) {
    while (length > 0) {
        ssize_t written = write(traceFD, bytes, length);
        if (written > 0) {
            bytes += written;
            length -= (size_t)written;
            continue;
        }
        if (written < 0 && errno == EINTR) continue;
        observerFatal([NSString stringWithFormat:@"trace write failed: errno=%d", errno]);
    }
}

static void traceRecord(NSDictionary *record) {
    if (traceFD < 0) observerFatal(@"trace is not open");
    NSError *error = nil;
    NSData *data = [NSJSONSerialization dataWithJSONObject:record options:0 error:&error];
    if (!data) observerFatal([NSString stringWithFormat:@"trace JSON serialization failed: %@", error]);
    writeAll(data.bytes, data.length);
    uint8_t newline = '\n';
    writeAll((const char *)&newline, 1);
    if (fsync(traceFD) != 0) observerFatal([NSString stringWithFormat:@"trace fsync failed: errno=%d", errno]);
}

static NSString *requiredAbsolutePath(const char *name) {
    const char *value = getenv(name);
    if (!value || value[0] != '/') {
        observerFatal([NSString stringWithFormat:@"%s must be an absolute path", name]);
    }
    return [NSString stringWithUTF8String:value];
}

static void openTrace(void) {
    NSString *path = requiredAbsolutePath("KCU_NATIVE_IME_TRACE_PATH");
    traceFD = open(path.fileSystemRepresentation, O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC, 0600);
    if (traceFD < 0) observerFatal([NSString stringWithFormat:@"cannot create trace with O_EXCL: %@", path]);
    traceRecord(@{@"kind": @"observer-loaded"});
}

static BOOL responderIsTextClient(id responder) {
    return responder && [responder isKindOfClass:[NSView class]] &&
           [responder respondsToSelector:@selector(hasMarkedText)] &&
           [responder respondsToSelector:@selector(inputContext)];
}

static NSView *textClientForWindow(NSWindow *window) {
    id responder = window.firstResponder;
    if (responderIsTextClient(responder) && ((NSView *)responder).window == window) return responder;
    id current = [NSTextInputContext currentInputContext].client;
    if (responderIsTextClient(current) && ((NSView *)current).window == window) return current;
    return nil;
}

static NSString *copyCallbackText(id value) {
    NSString *text = nil;
    if ([value isKindOfClass:[NSString class]]) {
        text = [value copy];
    } else if ([value isKindOfClass:[NSAttributedString class]]) {
        text = [[(NSAttributedString *)value string] copy];
    } else {
        observerFatal(@"text input callback payload is neither NSString nor NSAttributedString");
    }
    if (text.length > 128) observerFatal(@"text input callback payload exceeds 128 UTF-16 units");
    return text;
}

static void addRect(NSMutableDictionary *record, NSString *key, NSRect rect) {
    record[key] = @{ @"x": @(rect.origin.x), @"y": @(rect.origin.y),
                     @"width": @(rect.size.width), @"height": @(rect.size.height) };
}

static NSDictionary *rangeJSON(NSRange range) {
    return @{@"location": @(range.location), @"length": @(range.length)};
}

static void recordClientState(NSMutableDictionary *record, NSView *client) {
    if (!client || ![client respondsToSelector:@selector(hasMarkedText)]) {
        record[@"native_has_marked"] = [NSNull null];
        return;
    }
    record[@"native_has_marked"] = @([(id)client hasMarkedText]);
    if (![client respondsToSelector:@selector(markedRange)]) return;
    NSRange range = [(id<NSTextInputClient>)client markedRange];
    record[@"marked_range_location"] = @(range.location);
    record[@"marked_range_length"] = @(range.length);
    if (range.location == NSNotFound || ![client respondsToSelector:@selector(firstRectForCharacterRange:actualRange:)]) return;
    NSRange actualRange = NSMakeRange(NSNotFound, 0);
    NSRect firstRect = [(id<NSTextInputClient>)client firstRectForCharacterRange:range actualRange:&actualRange];
    record[@"first_rect_actual_location"] = @(actualRange.location);
    record[@"first_rect_actual_length"] = @(actualRange.length);
    addRect(record, @"first_rect", firstRect);
    if (client.window) {
        addRect(record, @"window_content_rect", [client.window contentRectForFrameRect:client.window.frame]);
        addRect(record, @"view_bounds", client.bounds);
    }
}

static void recordTextClientCallback(NSString *kind, NSString *text, NSView *client,
                                     NSRange selectedRange, NSRange replacementRange, BOOL hasArgumentRanges) {
    NSMutableDictionary *record = [@{ @"kind": kind, @"text": text,
                                      @"callback_uptime": @([NSProcessInfo processInfo].systemUptime) } mutableCopy];
    if (hasArgumentRanges) {
        record[@"callback_selected_range"] = rangeJSON(selectedRange);
        record[@"callback_replacement_range"] = rangeJSON(replacementRange);
    }
    recordClientState(record, client);
    traceRecord(record);
}

static void observedSetMarkedText(id self, SEL selector, id value, NSRange selectedRange, NSRange replacementRange) {
    if (!originalSetMarkedText) observerFatal(@"original setMarkedText IMP is unavailable");
    if (self != observedTextClient || traceFD < 0) {
        ((void (*)(id, SEL, id, NSRange, NSRange))originalSetMarkedText)(self, selector, value, selectedRange, replacementRange);
        return;
    }
    NSString *text = copyCallbackText(value);
    ((void (*)(id, SEL, id, NSRange, NSRange))originalSetMarkedText)(self, selector, value, selectedRange, replacementRange);
    recordTextClientCallback(@"native-preedit", text, (NSView *)self, selectedRange, replacementRange, YES);
}

static void observedInsertText(id self, SEL selector, id value, NSRange replacementRange) {
    if (!originalInsertText) observerFatal(@"original insertText IMP is unavailable");
    if (self != observedTextClient || traceFD < 0) {
        ((void (*)(id, SEL, id, NSRange))originalInsertText)(self, selector, value, replacementRange);
        return;
    }
    NSString *text = copyCallbackText(value);
    ((void (*)(id, SEL, id, NSRange))originalInsertText)(self, selector, value, replacementRange);
    recordTextClientCallback(@"native-insert", text, (NSView *)self, NSMakeRange(NSNotFound, 0), replacementRange, YES);
}

static void observedUnmarkText(id self, SEL selector) {
    if (!originalUnmarkText) observerFatal(@"original unmarkText IMP is unavailable");
    ((void (*)(id, SEL))originalUnmarkText)(self, selector);
    if (self == observedTextClient && traceFD >= 0) {
        recordTextClientCallback(@"native-unmark", @"", (NSView *)self,
                                 NSMakeRange(NSNotFound, 0), NSMakeRange(NSNotFound, 0), NO);
    }
}

static void installTextClientObserver(NSView *client) {
    if (textClientObserverInstalled) observerFatal(@"text input client observer was installed twice");
    Class clientClass = object_getClass(client);
    Method markedMethod = class_getInstanceMethod(clientClass, @selector(setMarkedText:selectedRange:replacementRange:));
    Method insertMethod = class_getInstanceMethod(clientClass, @selector(insertText:replacementRange:));
    Method unmarkMethod = class_getInstanceMethod(clientClass, @selector(unmarkText));
    if (!markedMethod || !insertMethod || !unmarkMethod) observerFatal(@"NSTextInputClient callback method is unavailable");
    originalSetMarkedText = method_getImplementation(markedMethod);
    originalInsertText = method_getImplementation(insertMethod);
    originalUnmarkText = method_getImplementation(unmarkMethod);
    if (originalSetMarkedText == (IMP)observedSetMarkedText ||
        originalInsertText == (IMP)observedInsertText || originalUnmarkText == (IMP)observedUnmarkText) {
        observerFatal(@"NSTextInputClient callback already points to observer");
    }
    observedTextClient = client;
    class_replaceMethod(clientClass, @selector(setMarkedText:selectedRange:replacementRange:),
                        (IMP)observedSetMarkedText, method_getTypeEncoding(markedMethod));
    class_replaceMethod(clientClass, @selector(insertText:replacementRange:),
                        (IMP)observedInsertText, method_getTypeEncoding(insertMethod));
    class_replaceMethod(clientClass, @selector(unmarkText), (IMP)observedUnmarkText,
                        method_getTypeEncoding(unmarkMethod));
    textClientObserverInstalled = YES;
}

static void traceModeError(NSString *message) {
    traceRecord(@{@"kind": @"mode-error", @"error": message ?: @"unknown error"});
}

static void prepareInputContext(void) {
    NSWindow *window = NSApp.keyWindow;
    NSView *responder = textClientForWindow(window);
    if (!window || !responder) {
        traceRecord(@{@"kind": @"mode-error", @"error": @"key window has no active NSTextInputClient",
                      @"first_responder_class": window.firstResponder ? NSStringFromClass([window.firstResponder class]) : @"<nil>"});
        observerFatal(@"input context prepare failed: first responder is not NSTextInputClient");
    }
    NSTextInputContext *context = [(NSView *)responder inputContext];
    if (!context) {
        traceModeError(@"own input context is unavailable");
        observerFatal(@"input context prepare failed: context unavailable");
    }
    BOOL markedBeforePrepare = [(id)responder hasMarkedText];
    if (markedBeforePrepare) {
        traceModeError(@"first responder already has marked text before input source prepare");
        observerFatal(@"input context prepare refused while marked text is active");
    }
    installSendEventObserver();
    // 入力方式は専用runnerの本物の入力メニューで選択し、ここでは観測だけ行う。
    NSTextInputContext *current = [NSTextInputContext currentInputContext];
    BOOL selected = [context.selectedKeyboardInputSource isEqualToString:[NSString stringWithUTF8String:kExpectedSource]];
    BOOL currentClient = current && current.client == (id)responder;
    BOOL marked = [(id)responder hasMarkedText];
    BOOL clientMatchesWindow = currentClient && responder.window == window;
    BOOL active = NSApp.isActive;
    BOOL key = window.isKeyWindow;
    BOOL visible = window.isVisible;
    if (!selected || !currentClient || !clientMatchesWindow || marked || !active || !key || !visible) {
        traceRecord(@{@"kind": @"mode-error", @"error": @"input context prepare did not reach the required state",
                      @"selected_input_source": context.selectedKeyboardInputSource ?: @"",
                      @"selected_matches": @(selected), @"current_client_matches": @(currentClient),
                      @"client_matches_window": @(clientMatchesWindow), @"actual_has_marked": @(marked),
                      @"app_active": @(active), @"window_key": @(key), @"window_visible": @(visible),
                      @"responder_class": NSStringFromClass([responder class]),
                      @"allowed_locales": context.allowedInputSourceLocales ?: (id)[NSNull null]});
        observerFatal(@"input context prepare did not reach the required state");
    }
    installTextClientObserver(responder);
    NSMutableDictionary *ready = [@{@"kind": @"mode-ready",
                  @"selected_input_source": context.selectedKeyboardInputSource ?: @"",
                  @"selected_matches": @(selected),
                  @"current_client_matches": @(currentClient),
                  @"client_matches_window": @(clientMatchesWindow),
                  @"app_active": @(active), @"window_key": @(key), @"window_visible": @(visible),
                  @"actual_has_marked": @(marked)} mutableCopy];
    addRect(ready, @"window_content_rect", [window contentRectForFrameRect:window.frame]);
    traceRecord(ready);
    modeReady = YES;
}

static void recordProcessedKey(NSEvent *event, NSString *eventType, NSTimeInterval handlerEnd) {
    NSWindow *window = event.window;
    id responder = window.firstResponder;
    NSNumber *marked = nil;
    NSNumber *clientMatchesWindow = nil;
    NSView *responderView = [responder isKindOfClass:[NSView class]] ? responder : nil;
    if (window && responderView && responderView.window == window && [responderView respondsToSelector:@selector(hasMarkedText)]) {
        marked = @([(id)responderView hasMarkedText]);
        clientMatchesWindow = @YES;
    } else {
        NSTextInputContext *current = [NSTextInputContext currentInputContext];
        id currentClient = current.client;
        NSView *currentView = [currentClient isKindOfClass:[NSView class]] ? currentClient : nil;
        if (window && currentView && currentView.window == window && [currentView respondsToSelector:@selector(hasMarkedText)]) {
            marked = @([(id)currentView hasMarkedText]);
            clientMatchesWindow = @YES;
        } else {
            clientMatchesWindow = @NO;
        }
    }
    NSMutableDictionary *record = [@{@"kind": @"key-processed",
                                     @"event_type": eventType,
                                     @"key_code": @(event.keyCode),
                                     @"event_timestamp": @(event.timestamp),
                                     @"handler_end_uptime": @(handlerEnd),
                                     @"client_matches_window": clientMatchesWindow} mutableCopy];
    NSView *textClient = textClientForWindow(window);
    if (textClient) {
        NSTextInputContext *context = [textClient inputContext];
        record[@"input_context_source"] = context.selectedKeyboardInputSource ?: @"";
        recordClientState(record, textClient);
    } else {
        record[@"native_has_marked"] = marked ?: [NSNull null];
    }
    traceRecord(record);
}

static void observedSendEvent(id self, SEL selector, NSEvent *event) {
    if (!originalSendEvent) observerFatal(@"original NSApplication sendEvent IMP is unavailable");
    // 元の処理が返った後だけ観測し、非同期の事前monitorを完了通知に使わない。
    ((void (*)(id, SEL, NSEvent *))originalSendEvent)(self, selector, event);
    if (modeReady && (event.type == NSEventTypeKeyDown || event.type == NSEventTypeKeyUp)) {
        recordProcessedKey(event, event.type == NSEventTypeKeyDown ? @"keyDown" : @"keyUp",
                           [NSProcessInfo processInfo].systemUptime);
    }
}

static void installSendEventObserver(void) {
    if (sendEventObserverInstalled) return;
    Class appClass = [NSApp class];
    SEL selector = @selector(sendEvent:);
    Method method = class_getInstanceMethod(appClass, selector);
    if (!method) observerFatal(@"NSApplication sendEvent method is unavailable");
    originalSendEvent = method_getImplementation(method);
    if (originalSendEvent == (IMP)observedSendEvent) observerFatal(@"NSApplication sendEvent already points to observer");
    class_replaceMethod(appClass, selector, (IMP)observedSendEvent, method_getTypeEncoding(method));
    sendEventObserverInstalled = YES;
}

static void prepareTimerFired(NSTimer *timer) {
    (void)timer;
    if (modeReady) return;
    NSString *path = preparePath;
    if (![[NSFileManager defaultManager] fileExistsAtPath:path]) return;
    NSWindow *window = NSApp.keyWindow;
    if (!NSApp.isActive || !window.isKeyWindow || !window.isVisible || !textClientForWindow(window)) {
        NSString *state = [NSString stringWithFormat:@"%d/%d/%d/%@", NSApp.isActive, window.isKeyWindow,
                           window.isVisible, window.firstResponder ? NSStringFromClass([window.firstResponder class]) : @"<nil>"];
        if (![state isEqualToString:lastPreparePendingState]) {
            traceRecord(@{@"kind": @"mode-pending", @"state": state});
            lastPreparePendingState = state;
        }
        return;
    }
    [prepareTimer invalidate];
    prepareTimer = nil;
    prepareInputContext();
}

static void didFinishLaunching(NSNotification *notification) {
    (void)notification;
    if (prepareTimer || modeReady) return;
    prepareTimer = [NSTimer timerWithTimeInterval:0.05 repeats:YES block:^(NSTimer *timer) {
        prepareTimerFired(timer);
    }];
    [[NSRunLoop mainRunLoop] addTimer:prepareTimer forMode:NSRunLoopCommonModes];
}

static void didTerminate(NSNotification *notification) {
    (void)notification;
    if (!modeReady) {
        traceModeError(@"prepare request did not arrive before application termination");
        observerFatal(@"observer terminated without mode-ready");
    }
    if (traceFD >= 0) close(traceFD);
    traceFD = -1;
}

__attribute__((constructor))
static void nativeObserverConstructor(void) {
    const char *actions = getenv("GITHUB_ACTIONS");
    const char *runnerOS = getenv("RUNNER_OS");
    const char *expectedBinary = getenv("KCU_NATIVE_IME_EXPECTED_BINARY");
    const char *program = getprogname();
    if (!actions || strcmp(actions, "true") != 0 || !runnerOS || strcmp(runnerOS, "macOS") != 0 ||
        !expectedBinary || !program || strcmp(program, expectedBinary) != 0) {
        return;
    }
    @autoreleasepool {
    // NSAppの生成は各backendに任せ、検証側が先に共有instanceを作らない。
        openTrace();
        preparePath = requiredAbsolutePath("KCU_NATIVE_IME_PREPARE_PATH");
        [[NSNotificationCenter defaultCenter] addObserverForName:NSApplicationDidFinishLaunchingNotification
                                                           object:nil
                                                            queue:[NSOperationQueue mainQueue]
                                                       usingBlock:^(NSNotification *note) {
            didFinishLaunching(note);
        }];
        [[NSNotificationCenter defaultCenter] addObserverForName:NSApplicationWillTerminateNotification
                                                           object:nil
                                                            queue:[NSOperationQueue mainQueue]
                                                       usingBlock:^(NSNotification *note) {
            didTerminate(note);
        }];
    }
}
