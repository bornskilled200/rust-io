#! /bin/sh
# /etc/init.d/pms5003

### BEGIN INIT INFO
# Provides:          pms5003
# Required-Start:    $local_fs $network
# Required-Stop:     $local_fs $network
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Short-Description: Simple script to start a program at boot
### END INIT INFO

PIDFILE=/var/run/pms5003.pid
DAEMON=/usr/local/bin/pms5003
CHDIR=/opt/pms5003
NAME=pms5003
DESC=pms5003

. /lib/lsb/init-functions

start_pms5003() {
  # Start the daemon/service
  #
  # Returns:
  #   0 if daemon has been started
  #   1 if daemon was already running
  #   2 if daemon could not be started
  start-stop-daemon --start --quiet --pidfile $PID --make-pidfile --background --chdir $CHDIR --startas /bin/bash -- -c "exec $DAEMON" --test > /dev/null \
          || return 1
  start-stop-daemon --start --quiet --pidfile $PID --make-pidfile --background --chdir $CHDIR --startas /bin/bash -- -c "exec $DAEMON $DAEMON_OPTS > /var/log/pms5003.log 2>&1" \
          || return 2
}

stop_pms5003() {
  # Stops the daemon/service
  #
  # Return
  #   0 if daemon has been stopped
  #   1 if daemon was already stopped
  #   2 if daemon could not be stopped
  #   other if a failure occurred
  start-stop-daemon --stop --quiet --pidfile $PID --name $NAME
  RETVAL="$?"
  sleep 1
  return "$RETVAL"
}


case "$1" in
  start)
    log_daemon_msg "Starting $DESC" "$NAME"
    start_pms5003
    case "$?" in
            0|1) log_end_msg 0 ;;
            2)   log_end_msg 1 ;;
    esac
    ;;
  stop)
    log_daemon_msg "Stopping $DESC" "$NAME"
    stop_pms5003
    case "$?" in
            0|1) log_end_msg 0 ;;
            2)   log_end_msg 1 ;;
    esac
    ;;
  restart)
    log_daemon_msg "Restarting $DESC" "$NAME"

    # Check configuration before stopping nginx
    if ! test_config; then
      log_end_msg 1 # Configuration error
      exit $?
    fi

    stop_pms5003
    case "$?" in
      0|1)
        start_pms5003
        case "$?" in
          0) log_end_msg 0 ;;
          1) log_end_msg 1 ;; # Old process is still running
          *) log_end_msg 1 ;; # Failed to start
        esac
        ;;
      *)
        # Failed to stop
        log_end_msg 1
        ;;
    esac
    ;;

  *)
    echo "Usage: /etc/init.d/pms5003 {start|stop|restart}"
    exit 1
    ;;
esac

exit 0
