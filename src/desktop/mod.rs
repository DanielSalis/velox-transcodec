mod intake;
mod view;
#[cfg(velox_qt_resize)]
mod window_resize;

use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Duration};

use notify_rust::{Hint, Notification};
use slint::{CloseRequestResponse, ComponentHandle, Weak};
use tokio::runtime::Handle;
use velox_transcode::{APP_ID, domain::BatchSummary, media::MediaEngine, queue};

use crate::DesktopWindow;
use view::Desktop;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .backend_name("qt".into())
        .select()?;
    slint::set_xdg_app_id(APP_ID)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    let worker = runtime.handle().clone();
    let queue::Queue {
        commands,
        mut events,
    } = {
        let _guard = runtime.enter();
        queue::spawn(MediaEngine::default())
    };

    let window = DesktopWindow::new()?;
    let state = Rc::new(RefCell::new(Desktop::new(&window, commands)));
    wire_callbacks(&window, &state, &worker);
    #[cfg(velox_qt_resize)]
    window_resize::wire(&window);

    let weak = window.as_weak();
    let ui_state = state.clone();
    let ui_worker = worker.clone();
    slint::spawn_local(async move {
        while let Some(event) = events.recv().await {
            let Some(window) = weak.upgrade() else { break };
            if let Some(summary) = ui_state.borrow_mut().apply(&window, event) {
                notify_finished(&ui_worker, summary);
            }
        }
        if let Some(window) = weak.upgrade() {
            ui_state.borrow_mut().disconnected(&window);
        }
    })?;

    state.borrow_mut().detect(&window);
    let paths = std::env::args_os().skip(1).map(PathBuf::from).collect();
    state.borrow().add_files(&window, paths);
    window.run()?;
    Ok(())
}

fn wire_callbacks(window: &DesktopWindow, state: &Rc<RefCell<Desktop>>, worker: &Handle) {
    window.on_inspect({
        let (window, state) = refs(window, state);
        move |index| {
            if let Some(window) = window.upgrade() {
                state.borrow_mut().inspect(&window, index);
            }
        }
    });
    window.on_toggle_inclusion({
        let (window, state) = refs(window, state);
        move || {
            if let Some(window) = window.upgrade() {
                state.borrow_mut().toggle_inclusion(&window);
            }
        }
    });
    window.on_codec_changed({
        let (window, state) = refs(window, state);
        move || {
            if let Some(window) = window.upgrade() {
                state.borrow().select_codec(&window);
            }
        }
    });
    window.on_encoder_changed({
        let (window, state) = refs(window, state);
        move |index| {
            if let Some(window) = window.upgrade() {
                state.borrow().select_encoder(&window, index);
            }
        }
    });
    window.on_recover_audio({
        let (window, state) = refs(window, state);
        move || {
            if let Some(window) = window.upgrade() {
                state.borrow_mut().recover_audio(&window);
            }
        }
    });
    window.on_toggle_queue({
        let (window, state) = refs(window, state);
        move || {
            if let Some(window) = window.upgrade() {
                state.borrow_mut().toggle_queue(&window);
            }
        }
    });
    window.on_detect_encoders({
        let (window, state) = refs(window, state);
        move || {
            if let Some(window) = window.upgrade() {
                state.borrow_mut().detect(&window);
            }
        }
    });
    window.window().on_close_requested({
        let (window, state) = refs(window, state);
        move || match window.upgrade() {
            Some(window) if state.borrow_mut().defer_close(&window) => {
                CloseRequestResponse::KeepWindowShown
            }
            _ => CloseRequestResponse::HideWindow,
        }
    });

    window.on_has_files(|data| data.has_file_paths());
    window.on_receive_files({
        let (window, state) = refs(window, state);
        move |data| {
            window.upgrade().is_some_and(|window| {
                state
                    .borrow()
                    .add_files(&window, intake::dropped_paths(data))
            })
        }
    });
    window.on_pick_files({
        let (window, state) = refs(window, state);
        let worker = worker.clone();
        move || {
            pick(&window, &state, &worker, intake::Picker::Videos);
        }
    });
    window.on_pick_destination({
        let (window, state) = refs(window, state);
        let worker = worker.clone();
        move || {
            pick(&window, &state, &worker, intake::Picker::Destination);
        }
    });
}

fn refs(
    window: &DesktopWindow,
    state: &Rc<RefCell<Desktop>>,
) -> (Weak<DesktopWindow>, Rc<RefCell<Desktop>>) {
    (window.as_weak(), state.clone())
}

fn pick(
    weak: &Weak<DesktopWindow>,
    state: &Rc<RefCell<Desktop>>,
    worker: &Handle,
    picker: intake::Picker,
) {
    let Some(window) = weak.upgrade() else { return };
    if window.get_picking() {
        return;
    }

    window.set_picking(true);
    let weak = weak.clone();
    let state = state.clone();
    let selection = worker.spawn(intake::select(picker));
    if let Err(error) = slint::spawn_local(async move {
        let result = selection.await;
        let Some(window) = weak.upgrade() else { return };
        window.set_picking(false);
        match result {
            Ok(intake::Selection::Videos(paths)) => {
                state.borrow().add_files(&window, paths);
            }
            Ok(intake::Selection::Destination(Some(path))) => {
                state.borrow_mut().set_destination(&window, path);
            }
            Ok(intake::Selection::Destination(None)) => (),
            Err(error) => {
                eprintln!("Falha no seletor: {error}");
                window.set_queue_message("O seletor do sistema falhou. Tente novamente.".into());
            }
        }
    }) {
        window.set_picking(false);
        eprintln!("Falha ao acompanhar o seletor: {error}");
        window.set_queue_message("O seletor do sistema falhou. Tente novamente.".into());
    }
}

fn notify_finished(worker: &Handle, summary: BatchSummary) {
    worker.spawn(async move {
        let mut notification = Notification::new();
        notification
            .appname("Velox Transcode")
            .icon(APP_ID)
            .hint(Hint::DesktopEntry(APP_ID.into()))
            .summary("Fila de conversão finalizada")
            .body(&view::batch_summary(summary));
        match tokio::time::timeout(Duration::from_secs(3), notification.show_async()).await {
            Ok(Ok(_)) => (),
            Ok(Err(error)) => eprintln!("Notificação do Velox indisponível: {error}"),
            Err(_) => eprintln!("O serviço de notificações não respondeu."),
        }
    });
}
