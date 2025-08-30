# Ultra-optimized STUN/TURN server for flight sim P2P
FROM alpine:latest

# install coturn (high-performance STUN/TURN server)
RUN apk add --no-cache coturn openssl

# create directories
RUN mkdir -p /var/lib/turn /var/log /etc/ssl/certs /etc/ssl/private

# generate self-signed cert for development
RUN openssl req -x509 -newkey rsa:2048 -keyout /etc/ssl/private/key.pem \
    -out /etc/ssl/certs/cert.pem -days 365 -nodes \
    -subj "/C=US/ST=CA/L=SF/O=FlightSim/OU=P2P/CN=stun.flightsim.local"

# copy config
COPY turnserver.conf /etc/turnserver.conf

# set permissions
RUN chown -R turnserver:turnserver /var/lib/turn /var/log
RUN chmod 600 /etc/ssl/private/key.pem
RUN chmod 644 /etc/ssl/certs/cert.pem

# expose ports
EXPOSE 3478/udp 3478/tcp 5349/udp 5349/tcp 10000-20000/udp

# health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD nc -zu localhost 3478 || exit 1

# run as turnserver user
USER turnserver

# start coturn with optimized config
CMD ["turnserver", "-c", "/etc/turnserver.conf", "--no-daemon", "--verbose"]