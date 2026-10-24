FROM kcu-screenshot-linux-arm64:local

RUN apt-get update && apt-get install -y --no-install-recommends \
    ibus ibus-mozc ibus-gtk3 dbus-x11 python3-gi python3-dbus \
    && useradd --create-home --uid 10001 kcu-ime

RUN mkdir -p /tmp/.X11-unix && chmod 1777 /tmp/.X11-unix

USER kcu-ime
ENV HOME=/home/kcu-ime \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8 \
    XMODIFIERS=@im=ibus \
    GTK_IM_MODULE=ibus \
    QT_IM_MODULE=ibus
