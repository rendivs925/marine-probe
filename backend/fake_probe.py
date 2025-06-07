import socket
import time

sock = socket.create_connection(('127.0.0.1', 9000))
while True:
    sock.sendall(b"temp=22.5,sal=31.0,turb=4.2\n")
    time.sleep(2)
