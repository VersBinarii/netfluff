# Netfluff

Netfluff monitors a remote IP endpoint and generates a notification when the latency increases above certain trachhold

# Config
It looks for a `~/.config/netfluff.toml` when upon start. If the config file cannot 
found it will load the default settings:
```
ping_dst = "google.pl"
check_freq = 60
warning_threshold = 70
```

Configure your `~/.config/sway/config` to start it upn launch with:
```
exec /path/to/netfluff
```
