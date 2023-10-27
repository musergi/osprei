pub async fn execute(source: String) -> Result<(), Error> {
    let engine = Engine::new().unwrap();
    engine
        .with_volume(|engine, volume| async move {
            if !engine
                .run(
                    vec!["git", "clone", &source, "code"],
                    "/workspaces",
                    volume.name(),
                )
                .await?
            {
                return Err(Error::Checkout);
            }
            if !engine
                .run(vec!["cargo", "test"], "/workspaces/code", volume.name())
                .await?
            {
                return Err(Error::Execution);
            }
            Ok(())
        })
        .await
}

#[derive(Clone)]
struct Engine {
    docker: docker_api::Docker,
}

impl Engine {
    fn new() -> Result<Engine, Error> {
        let docker_url = "unix:///var/run/docker.sock";
        log::info!("Connecting docker: {}", docker_url);
        let docker = docker_api::Docker::new(docker_url)?;
        Ok(Engine { docker })
    }

    async fn with_volume<F, Fut>(&self, action: F) -> Result<(), Error>
    where
        F: std::ops::FnOnce(Self, docker_api::Volume) -> Fut,
        Fut: std::future::Future<Output = Result<(), Error>>,
    {
        log::info!("Creating volume");
        let volume_ref = self.docker.volumes().create(&Default::default()).await?;
        let volume = docker_api::Volume::new(self.docker.clone(), volume_ref.name.clone());
        log::info!("Created volume: {}", volume.name());
        let result = action(self.clone(), volume).await;
        let volume = docker_api::Volume::new(self.docker.clone(), volume_ref.name);
        volume.delete().await?;
        log::info!("Deleted volume: {}", volume.name());
        result
    }

    async fn run<S>(
        &self,
        command: impl IntoIterator<Item = S>,
        working_dir: impl serde::Serialize,
        volume: impl std::fmt::Display,
    ) -> Result<bool, Error>
    where
        S: serde::Serialize,
    {
        let opts = docker_api::opts::ContainerCreateOpts::builder()
            .image("rust:latest")
            .volumes(vec![format!("{}:/workspaces", volume)])
            .working_dir(working_dir)
            .command(command)
            .build();
        let container = self.docker.containers().create(&opts).await?;
        log::info!("Created container: {}", container.id());
        container.start().await?;
        log::info!("Started container: {}", container.id());
        log::info!("Waiting container: {}", container.id());
        let success = container.wait().await?.status_code == 0;
        container.delete().await?;
        log::info!("Deleted container: {}", container.id());
        Ok(success)
    }
}

#[derive(Debug)]
pub enum Error {
    Docker(docker_api::Error),
    Checkout,
    Execution,
}

impl From<docker_api::Error> for Error {
    fn from(value: docker_api::Error) -> Error {
        Error::Docker(value)
    }
}
