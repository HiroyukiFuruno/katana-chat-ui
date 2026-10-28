ARG KCU_SCREENSHOT_IMAGE=kcu-screenshot-linux-arm64:local
FROM ${KCU_SCREENSHOT_IMAGE}

RUN apt-get update && apt-get install -y --no-install-recommends \
    ibus ibus-mozc ibus-gtk3 dbus-x11 python3-gi python3-dbus \
    && useradd --create-home --uid 10001 kcu-ime

# 親イメージのタグや後続apt更新に依存せず、標準CAストアの最低修正版を確認する。
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates openssl python3 curl \
    && dpkg --compare-versions "$(dpkg-query -W -f='${Version}' ca-certificates)" ge '20250419~deb12u2' \
    && update-ca-certificates --fresh

RUN mkdir -p /tmp/.X11-unix && chmod 1777 /tmp/.X11-unix

USER kcu-ime
ENV HOME=/home/kcu-ime \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8 \
    XMODIFIERS=@im=ibus \
    GTK_IM_MODULE=ibus \
    QT_IM_MODULE=ibus
