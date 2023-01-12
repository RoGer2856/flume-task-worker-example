use std::time::Duration;

use flume::Sender;
use tokio::{sync::oneshot, time};

struct ArithmeticAddTask {
    lhs: i64,
    rhs: i64,
    result_sender: oneshot::Sender<i64>,
}

async fn execute_slow_task(task: ArithmeticAddTask) {
    time::sleep(Duration::from_secs(1)).await;

    task.result_sender.send(task.lhs + task.rhs).unwrap();
}

async fn send_and_wait_task(task_sender: Sender<ArithmeticAddTask>) {
    let (result_sender, result_receiver) = oneshot::channel();

    task_sender
        .send(ArithmeticAddTask {
            lhs: 30,
            rhs: 12,
            result_sender,
        })
        .unwrap();

    let result = result_receiver.await.unwrap();
    println!("result = {result}");
}

#[tokio::main]
async fn main() {
    let (task_sender, task_receiver) = flume::unbounded();

    let worker = tokio::spawn(async move {
        while let Ok(task) = task_receiver.recv_async().await {
            // the task is slow to make sure that there are more tasks in the queue than the worker can handle
            execute_slow_task(task).await;
        }
    });

    let task_sender_0 = task_sender.clone();
    let task_sender_1 = task_sender;

    let task_0 = tokio::spawn(send_and_wait_task(task_sender_0));
    let task_1 = tokio::spawn(send_and_wait_task(task_sender_1));

    // wait for half a second to make sure one of the tasks is already fetched by the worker
    time::sleep(Duration::from_millis(500)).await;

    // each task takes ~1 second so the worker thread could fetch only one task before aborting
    worker.abort();

    // one of the task is still in the channel's queue, though there is no worker that can execute it
    // therefore this task will wait forever for the answer
    let _ = task_0.await;
    let _ = task_1.await;
}
