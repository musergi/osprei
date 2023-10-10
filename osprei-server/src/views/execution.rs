use docker_api::conn::TtyChunk;
use docker_api::opts::ContainerCreateOpts;
use docker_api::opts::ContainerRemoveOpts;
use docker_api::opts::LogsOpts;
use docker_api::Containers;
use docker_api::Docker;
use tokio_stream::StreamExt;

use osprei::Job;
use osprei::JobPointer;

pub fn dispatch_execution(
    execution_id: i64,
    pointer: JobPointer,
    storage: super::Storage,
    docker: Docker,
) {
    tokio::spawn(async move {
        let JobPointer {
            name, definition, ..
        } = pointer;
        let Job {
            source,
            image,
            command,
            arguments,
            ..
        } = definition;
        let checkout_path = format!("/opt/osprei/var/workspace/{}", name);
        tokio::process::Command::new("git")
            .arg("clone")
            .arg(source)
            .arg(&checkout_path)
            .output()
            .await
            .unwrap();
        let volume = format!("{v}:{v}", v = checkout_path);
        let mut command = vec![command];
        command.extend(arguments);
        let opts = ContainerCreateOpts::builder()
            .name(&name)
            .image(image)
            .working_dir(&checkout_path)
            .command(command)
            .volumes(vec![volume])
            .build();
        let container = Containers::new(docker).create(&opts).await.unwrap();
        container.start().await.unwrap();
        let status = container.wait().await.unwrap().status_code;
        let status = if status == 0 { 0 } else { 1 };
        let opts = LogsOpts::builder().stdout(true).stderr(true).build();
        let mut stream = container.logs(&opts);
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        while let Some(text) = stream.next().await {
            match text.unwrap() {
                TtyChunk::StdOut(out) => stdout.extend(out),
                TtyChunk::StdErr(out) => stderr.extend(out),
                _ => (),
            }
        }
        let stdout = String::from_utf8_lossy(&stdout).to_string();
        let stderr = String::from_utf8_lossy(&stderr).to_string();
        storage
            .update_execution(execution_id, status, stdout, stderr)
            .await
            .unwrap();
        let opts = ContainerRemoveOpts::builder()
            .force(true)
            .volumes(true)
            .build();
        container.remove(&opts).await.unwrap();
    });
}
