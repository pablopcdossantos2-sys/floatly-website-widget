use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

#[cfg(windows)]
use windows::{
    ApplicationModel::{Package, StartupTask, StartupTaskState},
    core::HSTRING,
};
#[cfg(windows)]
use windows_future::*;

/// The startup task declared in Package.appxmanifest for the Store/MSIX build.
#[cfg(windows)]
const MSIX_STARTUP_TASK_ID: &str = "FloatlyStartup";

#[cfg(windows)]
fn packaged_startup_task() -> Result<Option<StartupTask>, String> {
    // Package::Current fails for a normal unpackaged desktop process.
    if Package::Current().is_err() {
        return Ok(None);
    }

    let task_id = HSTRING::from(MSIX_STARTUP_TASK_ID);
    let operation = StartupTask::GetAsync(&task_id).map_err(|error| {
        format!("Não foi possível localizar a tarefa de inicialização MSIX: {error}")
    })?;
    let task = operation.join().map_err(|error| {
        format!("Não foi possível consultar a tarefa de inicialização MSIX: {error}")
    })?;

    Ok(Some(task))
}

#[cfg(windows)]
fn startup_state_enabled(state: StartupTaskState) -> bool {
    state == StartupTaskState::Enabled || state == StartupTaskState::EnabledByPolicy
}

/// Reports the actual OS state. Packaged builds use the MSIX StartupTask API;
/// unpackaged builds retain the existing Tauri autostart plugin behavior.
pub fn is_enabled(app: &AppHandle) -> Result<bool, String> {
    #[cfg(windows)]
    if let Some(task) = packaged_startup_task()? {
        let state = task.State().map_err(|error| {
            format!("Não foi possível ler o estado da inicialização MSIX: {error}")
        })?;
        return Ok(startup_state_enabled(state));
    }

    app.autolaunch()
        .is_enabled()
        .map_err(|error| format!("Não foi possível consultar a inicialização automática: {error}"))
}

/// Changes the startup preference and returns the state accepted by Windows.
pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    #[cfg(windows)]
    if let Some(task) = packaged_startup_task()? {
        if enabled {
            let state = task
                .RequestEnableAsync()
                .map_err(|error| {
                    format!("Não foi possível solicitar a inicialização MSIX: {error}")
                })?
                .join()
                .map_err(|error| {
                    format!("A solicitação de inicialização MSIX falhou: {error}")
                })?;
            return Ok(startup_state_enabled(state));
        }

        task.Disable()
            .map_err(|error| format!("Não foi possível desativar a inicialização MSIX: {error}"))?;
        return Ok(false);
    }

    if enabled {
        app.autolaunch()
            .enable()
            .map_err(|error| format!("Não foi possível ativar a inicialização automática: {error}"))?;
    } else {
        app.autolaunch()
            .disable()
            .map_err(|error| format!("Não foi possível desativar a inicialização automática: {error}"))?;
    }

    app.autolaunch()
        .is_enabled()
        .map_err(|error| format!("Não foi possível confirmar a inicialização automática: {error}"))
}
