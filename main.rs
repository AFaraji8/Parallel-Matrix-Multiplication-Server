use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{thread, vec};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

#[derive(Debug, Serialize,Clone)]
struct Matrix{
    data: Vec<Vec<f64>>,
}

impl Matrix {
    fn new(rows: usize, columns: usize, init_value: f64) -> Matrix 
    where
    f64: Clone,
    {
        let data: Vec<Vec<f64>> = vec![vec![init_value; columns]; rows];
        Matrix { data }
    }

    fn rows(&self) -> usize {
        self.data.len()
    }

    fn columns(&self) -> usize {
        if let Some(row) = self.data.get(0) {
            row.len()
        } else {
            0
        }
    }

    fn newcopy (&mut self,m:Matrix) {

        for i in 0.. m.rows() {
            for j in 0.. m.columns() {
                let w=match m.get(i, j) {
                    Some(r)=>*r,
                    None=>0.0,  
                };
                self.set(i, j,w );
            }
        }
    }


    fn get(&self, row: usize, column: usize) -> Option<&f64> {
        self.data.get(row).and_then(|r| r.get(column))
    }

    fn set(&mut self, row: usize, column: usize, value: f64) {
        if let Some(r) = self.data.get_mut(row) {
            if let Some(c) = r.get_mut(column) {
                *c = value;
            }
        }
    }

    fn get_row(&self, row: usize) -> Vec<f64> {

        let mut result: Vec<f64>= Vec::new();
        for e in 0..self.columns() {
            let ee=self.get(row, e);
            match ee {
                Some(q) => {result.push(*q);}
                None=>{print!("");}    
            } 
        
            
        }
        return result;

    }

    fn get_column(&self, column: usize) ->Vec<f64> {
        let mut result = Vec::new();
        for row in &self.data {
            if let Some(value) = row.get(column) {
                result.push(*value);
            }
        }
        return result;
        } 
}


struct Request{
    id:usize,
    matrix1 : Matrix,
    matrix2 :Matrix,

}



struct Requestworker{
    id:usize,
    rowm1:Vec<f64>,
    rownumber:usize,
    column2:Vec<f64>,
    columnnumber:usize,

}

struct Responseworker{
    id:usize,
    rownumber:usize,
    columnnumber:usize,
    value:f64,
}


fn Multiplicationv(rq:Requestworker) ->Responseworker
{
    let r=rq.rowm1;
    let c=rq.column2;
    let mut sum=0.0;
    for i in 0..r.len()  {
        sum+=r[i]*c[i];
    }
    let result=Responseworker { 
        id: (rq.id),
         rownumber: (rq.rownumber),
          columnnumber: (rq.columnnumber),
           value: (sum) };
    return result;



}


struct Worker{
    sem:bool,
}

impl Worker {

    fn state(&self)-> bool
    {
        return self.sem;
    }

    fn change(& mut self)
    {
        self.sem= !self.sem;
    }
    fn action(&self,Requestworker : Requestworker,  resultmatrix: Arc<Mutex<Matrix>>) -> Responseworker
    {

        let mut resultm = resultmatrix.lock().unwrap();

        let x=Multiplicationv(Requestworker);
        resultm.set(x.rownumber, x.columnnumber, x.value);
        return x;

        
    }


}


struct QueueWorkers{
    queue:Vec<Worker>,
}

impl QueueWorkers {
    fn new() -> Self {
        QueueWorkers { queue: Vec::new() }
    }


    fn enqueue(&mut self, worker: Worker) -> bool {
            self.queue.push(worker);
            true
    }





    fn dequeue(&mut self) -> Option<Worker> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }


}



struct AQueueWorkers{
    queue:Vec<Arc<Mutex<Worker>>>,
}


impl AQueueWorkers {

fn enqueuee(&mut self, worker: Arc<Mutex<Worker>>) -> bool {
    self.queue.push(worker);
    true
}

fn new() -> Self {
    AQueueWorkers { queue: Vec::new() }
}
fn dequeue(&mut self) -> Option<Arc<Mutex<Worker>>> {
    if self.queue.is_empty() {
        None
    } else {
        Some(self.queue.remove(0))
    }
}

}

async fn dowork(matrix1:Matrix,matrix2:Matrix) -> Arc<Mutex<Matrix>>
{

    let matrix1=Arc::new(Mutex::new(matrix1));
    let matrix2=Arc::new(Mutex::new(matrix2));
    let satrr=matrix1.clone().try_lock().unwrap().rows();
    
    
    let mut handles= Vec::new();
    let (tx, rx) = mpsc::channel();
    let mut handle;
    
    let m2=matrix2.clone();
    let m1=matrix1.clone();
    let matrix1 = Arc::new(Mutex::new(matrix1));
    let matrix2 = Arc::new(Mutex::new(matrix2));
    
    for i in 0..m1.lock().unwrap().rows() {
    
        for j in 0..m2.lock().unwrap().columns() {
            let mm1=m1.clone();
            let mm2=m2.clone();
        let matrix1 = Arc::new(Mutex::new(mm1));
        let matrix2 = Arc::new(Mutex::new(mm2));
    
            let tx1 = tx.clone();
            handle=thread::spawn(move || {
            let rq= Requestworker{
                id:(i*1000)+j,
                rowm1:matrix1.clone().lock().unwrap().lock().unwrap().get_row(i),
                rownumber:i,
                column2:matrix2.clone().lock().unwrap().lock().unwrap().get_column(j),
                columnnumber:j,
    
            };
            tx1.send(rq).unwrap();
    
        });
        handles.push(handle);
        }    
    }
    
    for handlee in handles {
        handlee.join().unwrap();
    }
    
    

    let matrixresult = Arc::new(Mutex::new(Matrix::new(matrix1.clone().lock().unwrap().lock().unwrap().rows(),matrix2.clone().lock().unwrap().lock().unwrap().columns(), 0.0)));
    
    let qq = Arc::new(Arc::new(Mutex::new(AQueueWorkers::new())));
    for _i in  0..5 {
        let w=Worker{sem:true,};
        let counter = Arc::new(Mutex::new(w));
        qq.clone().lock().unwrap().enqueuee(counter);
    }
    let semaphore = Arc::new(Semaphore::new(5));
    
    
    
    
    
    let mut handles= Vec::new();
    
    for x in rx.try_iter() {
    
        let a=qq.clone();
        let b=qq.clone();
        let matrixresult = Arc::clone(&matrixresult);
        while qq.clone().lock().unwrap().queue.len()==0 {    }
        let k =qq.clone().lock().unwrap().dequeue();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        handles.push(tokio::spawn(async move {
            let r=match k {
                Some(r)=>r,
                None=> panic!("err"),
            };
            r.lock().unwrap().change();
            r.lock().unwrap().action(x, matrixresult);
            drop(permit);
            // println!("finished");
            r.lock().unwrap().change();
            b.lock().unwrap().enqueuee(r);
            
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    println!("result :");
    for i in 0..satrr{
        println!("{:?}", matrixresult.clone().lock().unwrap().get_row(i));  
    
    }
    return matrixresult.clone();
    
}




























#[derive(Debug, Deserialize)]
struct MatrixData {
    #[serde(rename = "matrixA")]
    matrix_a: Vec<Vec<f64>>,
    #[serde(rename = "matrixB")]
    matrix_b: Vec<Vec<f64>>,
}




fn multiply_matrices(a: &Matrix, b: &Matrix) -> Result<Matrix, String> {
    if a.columns() != b.rows() {
        return Err(format!(
            "incompatible matrix dimensions: {}x{} and {}x{}",
            a.rows(), a.columns(), b.rows(), b.columns()
        ));
    }

    let c_rows = a.rows();
    let c_cols = b.columns();
    let mut c_data = vec![vec![0.0; c_cols]; c_rows];

    for i in 0..c_rows {
        for j in 0..c_cols {
            let mut sum = 0.0;
            for k in 0..a.columns() {
                sum += a.data[i][k] * b.data[k][j];
            }
            c_data[i][j] = sum;
        }
    }

    Ok(Matrix {
        data: c_data,
    })
}



fn handle_client(mut stream: TcpStream) -> Result<(Matrix, Matrix), String> {
    let mut buffer = [0; 1024];
    let mut data = String::new();

    match stream.read(&mut buffer) {
        Ok(_) => {
            data.push_str(&String::from_utf8_lossy(&buffer).trim_end_matches(char::from(0)));
            match serde_json::from_str::<MatrixData>(&data) {
                Ok(matrix_data) => {
                    
                    let a = Matrix {
                        data: matrix_data.matrix_a.clone(),
                    };

                    let b = Matrix {

                        data: matrix_data.matrix_b.clone(),
                    };




                    match multiply_matrices(&a, &b) {
                        Ok(result) => {
                            let response = serde_json::to_string(&result).unwrap();
                            stream.write(response.as_bytes()).unwrap();
                            Ok((a, b))
                        }
                        Err(error) => {
                            let error_message = format!("{{\"error\":\"{}\"}}", error);
                            stream.write(error_message.as_bytes()).unwrap();
                            return Err(error_message)
                        }
                    }
                }
                Err(_) => {
                    let error_message = "{\"error\":\"Invalid JSON\"}";
                    stream.write(error_message.as_bytes()).unwrap();
                    return Err(error_message.into())
                }
            }
        }
        Err(_) => {
            let error_message = "{\"error\":\"Error reading data from client\"}";
            stream.write(error_message.as_bytes()).unwrap();
            return Err(error_message.into())
        }
    }
}







#[tokio::main]
async fn main() {



    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();
    println!("Server listening on port 9090");
    let (tx, rx) = mpsc::channel();

    for stream in listener.incoming() {
        let tx1 = tx.clone();
        let stream = stream.unwrap();

        let q=thread::spawn(move || {
           let qqq= handle_client(stream);

           let w: (Matrix, Matrix)=match qqq {
               Ok(w)=>{w}
               Err(_)=>(Matrix::new(1,1,0.0), Matrix::new(1,1,0.0)),

           };
           tx1.send(w).unwrap();
           


        });
        q.join().unwrap();
        break;
    }

    for x in rx.try_iter() {
    let aa=(x.0);
    println!("{:?}", aa.get_row(0)); 

    
    let bb=(x.1);
    println!("{:?}", bb.get_row(0));



    let result=dowork(aa, bb).await;
    // println!("{:?}", result.lock().unwrap().get_column(0));
        
}




}

