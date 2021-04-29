use std::{
    process::Stdio,
    sync::atomic::{AtomicUsize, Ordering},
};

use tokio::{
    io::AsyncBufReadExt,
    io::BufReader,
    task::{self, JoinHandle},
};

pub(crate) static GLOBAL_JOB_COUNT: AtomicUsize = AtomicUsize::new(0);

use tokio::process::Command;

pub struct ExperimentProcess {
    task: JoinHandle<()>,
}

impl ExperimentProcess {
    pub async fn new(command: String) -> Result<ExperimentProcess, anyhow::Error> {
        GLOBAL_JOB_COUNT.fetch_add(1, Ordering::SeqCst);
        let mut s = command.split_ascii_whitespace();
        let mut child = Command::new(s.next().unwrap())
            .args(s)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let task = task::spawn(async move {
            let res = child.wait().await.unwrap();
            let mut output = BufReader::new(child.stdout.take().expect("a")).lines();
            while let Some(line) = output.next_line().await.unwrap() {
                println!("Line: {}", line);
            }
            GLOBAL_JOB_COUNT.fetch_sub(1, Ordering::SeqCst);
            if res.success() {
                todo!()
            } else if res.code().unwrap() == 124 {
                todo!()
            }
        });
        Ok(ExperimentProcess { task })
    }
}

// pub(crate) struct ParallelProcess {
//     pub child: Child,
// }

// impl ParallelProcess {
//     pub async fn new(nb_jobs: usize) -> Result<Self, anyhow::Error> {
//         let mut child = Command::new("parallel")
//             // .arg("--pipe")
//             .arg("-j")
//             .arg(nb_jobs.to_string())
//             .stdin(Stdio::piped())
//             // .stdout(Stdio::piped())
//             .spawn()?;
//         // .and_then(|c| { c}).expect("failed to");

//         let child_stdin = child.stdin.as_mut().expect("failed to open stdin");
//         child_stdin.write_all("echo a\n".as_bytes()).await?;
//         child_stdin
//             .write_all("sleep 4; echo a\n".as_bytes())
//             .await?;
//         child_stdin
//             .write_all("sleep 4; echo a\n".as_bytes())
//             .await?;
//         child_stdin
//             .write_all("sleep 4; echo a\n".as_bytes())
//             .await?;
//         child_stdin
//             .write_all("sleep 4; echo a\n".as_bytes())
//             .await?;
//         child_stdin
//             .write_all("sleep 24; echo a\n".as_bytes())
//             .await?;
//         child_stdin
//             .write_all("sleep 34; echo a\n".as_bytes())
//             .await?;
//         child_stdin
//             .write_all("sleep 44; echo a\n".as_bytes())
//             .await?;
//         child_stdin
//             .write_all("sleep 15; echo a\n".as_bytes())
//             .await?;
//         child_stdin
//             .write_all("sleep 14; echo a\n".as_bytes())
//             .await?;

//         // let mut child_stdout = child.stdout.take().expect("failed to get stdout");
//         // let mut buf = vec![];
//         // let _ = child.stdout.take().unwrap().read_buf(&mut buf);
//         // println!("{:?}", buf);
//         // dbg!(child_stdout)

//         Ok(ParallelProcess { child })
//     }
// }
