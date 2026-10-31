# Yuna OS aliases for interactive shells
if [[ $- == *i* ]]; then
    alias fetch='fastfetch'
    alias welcome='yuna-welcome'
    alias yuna='yunainstall'
    alias wifi='yuna-wifi'
    alias net='nmtui'
    alias firewall='sudo ufw status verbose'
    alias update='sudo pacman -Syu'
    if [[ -z ${YUNA_BANNER_SHOWN:-} ]] && command -v fastfetch >/dev/null 2>&1; then
        export YUNA_BANNER_SHOWN=1
        fastfetch --config /etc/skel/.config/fastfetch/config.jsonc 2>/dev/null || true
    fi
fi
