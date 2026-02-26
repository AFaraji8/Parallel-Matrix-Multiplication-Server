# Parallel Matrix Multiplication Server (Rust & Python)

A high-performance, asynchronous TCP server environment implemented in **Rust** designed for distributed matrix operations. This system leverages **Shared-State Concurrency**, **Worker-Pool Patterns**, and **Asynchronous Task Scheduling** to optimize large-scale mathematical computations.

---

## 🎯 Program Objectives (Goal)
The primary objective is to create a scalable **Client-Server Architecture** for matrix multiplication. The workflow is designed as follows:
- **Distributed Computing**: Offload heavy matrix calculations from a Python client to a high-speed Rust server.
- **Parallel Task Decomposition**: Break down a single matrix multiplication into $N \times M$ independent atomic tasks (one for each cell in the result matrix).
- **Concurrency Control**: Utilize **Semaphores** to limit concurrent tasks, preventing CPU saturation and ensuring system stability.
- **Cross-Language Integration**: Seamless data exchange between Python and Rust using **JSON** over TCP.

---

## 🔑 Key Variable Definitions

### Data Structures
- **`Matrix`**: The core struct containing a 2D vector (`Vec<Vec<f64>>`). It encapsulates methods for row/column extraction and safe element access.
- **`Requestworker`**: A lightweight payload containing only the necessary row and column vectors required to compute a single cell value ($C_{ij} = \sum A_{ik} \times B_{kj}$).
- **`Worker`**: A stateful unit that tracks its availability using a boolean semaphore (`sem`).
- **`AQueueWorkers`**: An asynchronous, thread-safe queue (`Arc<Mutex<Vec>>`) that manages the lifecycle of available workers.

### Concurrency Tools
- **`Arc<Mutex<T>>`**: Used to share the result matrix and worker queue across multiple threads/tasks without data races.
- **`mpsc::channel`**: The communication backbone used to stream task requests from the distributor to the execution pool.
- **`tokio::sync::Semaphore`**: A synchronization primitive that limits the server to exactly **5 concurrent async tasks** at any given time.

---

## 🛠️ Function Definitions

### Core Engine
- **`main()`**: Initializes the TCP listener on port `9090`, handles incoming client streams, and orchestrates the hand-off between networking and computation.
- **`handle_client()`**: Decodes JSON payloads from the TCP stream, validates matrix dimensions, and returns an initial result or error message via `serde_json`.
- **`dowork()`**: The primary **Asynchronous Orchestrator**. It:
    1. Spawns threads to decompose the matrices into individual row/column pairs.
    2. Populates the `AQueueWorkers`.
    3. Uses a `tokio` loop to spawn parallel tasks that process the `mpsc` receiver stream.
- **`Multiplicationv()`**: The atomic unit of work. It performs the mathematical dot product of two vectors.

### Worker Logic
- **`Worker::action()`**: Locks the shared `matrixresult`, invokes the calculation, and updates the specific cell coordinates with the computed value.
- **`Worker::change()`**: Toggles the worker's state between "Busy" and "Available".

---

## 📤 Program Output
1. **Server Initialization**: `Server listening on port 9090` appears in the console.
2. **Data Verification**: Upon receiving a request, the server prints the first row of both input matrices to confirm successful deserialization.
3. **Parallel Trace**: The server prints the resulting rows as they are finalized by the worker pool.
4. **Client Response**: The Python script receives a JSON object (e.g., `{"data": [[...], [...]]}`) and prints it to the terminal.

---
