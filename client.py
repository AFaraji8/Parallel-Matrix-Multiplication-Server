import socket
import json


def send_request(host, port, matrixA, matrixB):
    data = {
        "matrixA": matrixA,
        "matrixB": matrixB
    }
    json_data = json.dumps(data)

    client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    client_socket.connect((host, port))

    client_socket.send(json_data.encode())

    response = client_socket.recv(1024).decode()

    print("Response from server:")
    print(response)

    client_socket.close()


matrixA = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 2, 1]
]

matrixB = [
    [1, 2, 7],
    [2, 4, 6],
    [7, 2, 1]
]

host = "127.0.0.1"
port = 9090

send_request(host, port, matrixA, matrixB)
