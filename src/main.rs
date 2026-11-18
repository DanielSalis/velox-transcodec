use std::{path::PathBuf, rc::Rc};

use rfd::AsyncFileDialog;
use slint::{ComponentHandle, DataTransfer, ModelRc, SharedString, VecModel};
use tokio::runtime::Runtime;
use velox_transcode::{APP_ID, domain::QueueCommand};

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .backend_name("qt".into())
        .select()?;
    slint::set_xdg_app_id(APP_ID)?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()?;
    let window = IntakeWindow::new()?;
    let rows = Rc::new(VecModel::default());
    window.set_files(ModelRc::from(rows.clone()));
    wire_intake(&window, rows.clone(), &runtime);
    add_files(
        QueueCommand::AddFiles {
            paths: std::env::args_os().skip(1).map(PathBuf::from).collect(),
        },
        &rows,
    );
    window.run()?;
    Ok(())
}

fn wire_intake(window: &IntakeWindow, rows: Rc<VecModel<SharedString>>, runtime: &Runtime) {
    window.on_has_files(|data| data.has_file_paths());
    window.on_receive_files({
        let rows = rows.clone();
        move |data| {
            add_files(
                QueueCommand::AddFiles {
                    paths: dropped_paths(data),
                },
                &rows,
            )
        }
    });

    let worker = runtime.handle().clone();
    let weak = window.as_weak();
    window.on_pick_files(move || {
        let Some(window) = weak.upgrade() else { return };
        window.set_picking(true);
        let weak = weak.clone();
        let rows = rows.clone();
        let selection = worker.spawn(pick_paths());
        if let Err(error) = slint::spawn_local(async move {
            let paths = selection.await.unwrap_or_else(|error| {
                eprintln!("Falha no seletor de arquivos: {error}");
                Vec::new()
            });
            if let Some(window) = weak.upgrade() {
                add_files(QueueCommand::AddFiles { paths }, &rows);
                window.set_picking(false);
            }
        }) {
            window.set_picking(false);
            eprintln!("Falha ao acompanhar o seletor de arquivos: {error}");
        }
    });
}

async fn pick_paths() -> Vec<PathBuf> {
    AsyncFileDialog::new()
        .set_title("Selecionar vídeos")
        .add_filter(
            "Vídeos",
            &[
                "mp4", "mkv", "webm", "mov", "avi", "m4v", "mpg", "mpeg", "ts",
            ],
        )
        .add_filter("Todos os arquivos", &["*"])
        .pick_files()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|file| file.path().to_path_buf())
        .collect()
}

fn dropped_paths(data: DataTransfer) -> Vec<PathBuf> {
    data.file_paths()
        .map(|paths| paths.map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}

fn add_files(command: QueueCommand, rows: &VecModel<SharedString>) -> bool {
    let QueueCommand::AddFiles { paths } = command else {
        return false;
    };
    let received = !paths.is_empty();
    for path in paths {
        rows.push(path.to_string_lossy().as_ref().into());
    }
    received
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_multiple_native_paths_in_order() {
        let expected = [
            PathBuf::from("/tmp/um vídeo.mp4"),
            PathBuf::from("/tmp/dois.mkv"),
        ];
        let mut transfer = DataTransfer::default();
        transfer.set_file_paths(&expected);
        assert_eq!(dropped_paths(transfer), expected);
    }

    #[test]
    fn rejects_plain_text_as_file_input() {
        let transfer = DataTransfer::from(SharedString::from("file:///tmp/video.mp4"));
        assert!(dropped_paths(transfer).is_empty());
    }
}
