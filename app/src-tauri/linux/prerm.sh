#!/bin/sh
# Botloft's .deb pre-removal script (spec 15.4). dpkg runs it as root
# before it removes the package, and before an upgrade replaces it. On a
# removal only, it stops Botloft's background service for every user who
# has one, as `botloftd service uninstall` does, and removes the entry that
# opens the app at sign-in. An upgrade (the in-app updater installs the
# new .deb) leaves both alone. Bots and data stay in each user's folders.
set -u

case "${1:-}" in
  remove | purge) ;;
  *) exit 0 ;;
esac

for unit in /home/*/.config/systemd/user/botloft.service /root/.config/systemd/user/botloft.service; do
  [ -f "$unit" ] || continue
  home=${unit%/.config/systemd/user/botloft.service}
  user=$(stat -c %U "$home") || continue
  uid=$(id -u "$user") || continue
  data="$home/.local/share/Botloft"
  daemon="$data/bin/botloftd"
  if [ -x "$daemon" ] && [ -S "/run/user/$uid/bus" ]; then
    # The user's service manager runs: the daemon stops its own service.
    runuser -u "$user" -- env HOME="$home" XDG_RUNTIME_DIR="/run/user/$uid" \
      DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$uid/bus" \
      "$daemon" --home "$data" service uninstall >/dev/null 2>&1 || true
  fi
  # Signed out, or the daemon could not: nothing runs it, so its files go.
  rm -f "$unit" "$home/.config/systemd/user/default.target.wants/botloft.service"
  rm -f "$home/.config/autostart/botloft.desktop"
done

exit 0
