mirakurunのチャンネルIDを参照して、``127.0.0.1:4000/stream/:channel_id``にアクセスすると視聴が可能になります。

メモ
``` shell-session
apt update && apt upgrade && apt install -y udev && apt install -y build-essential && apt install -y wget && apt install -y git && apt install -y gcc && apt install make && apt install -y automake && apt install -y autoconf && apt install -y libtool
wget https://github.com/tsukumijima/px4_drv/releases/download/v0.4.4/px4-drv-dkms_0.4.4_all.deb
apt install -y ./px4-drv-dkms_0.4.4_all.deb
```
