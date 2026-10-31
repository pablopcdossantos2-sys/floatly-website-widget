#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrayLabels {
    pub show_hide: &'static str,
    pub reload: &'static str,
    pub settings: &'static str,
    pub lock: &'static str,
    pub topmost: &'static str,
    pub login: &'static str,
    pub quit: &'static str,
    pub tooltip: &'static str,
}

pub fn tray_labels(language: &str, locked: bool) -> TrayLabels {
    match language {
        "en" => TrayLabels {
            show_hide: "Show / hide",
            reload: "Reload page",
            settings: "Settings",
            lock: if locked {
                "Unlock moving / resizing"
            } else {
                "Lock moving / resizing"
            },
            topmost: "Toggle always on top",
            login: "Sign in / configure website",
            quit: "Quit",
            tooltip: "Floatly — Website Widget",
        },
        "es" => TrayLabels {
            show_hide: "Mostrar / ocultar",
            reload: "Actualizar página",
            settings: "Configuración",
            lock: if locked {
                "Desbloquear movimiento / tamaño"
            } else {
                "Bloquear movimiento / tamaño"
            },
            topmost: "Alternar siempre visible",
            login: "Iniciar sesión / configurar sitio",
            quit: "Salir",
            tooltip: "Floatly — Ventana de sitio web",
        },
        _ => TrayLabels {
            show_hide: "Mostrar / ocultar",
            reload: "Recarregar página",
            settings: "Configurações",
            lock: if locked {
                "Destravar mover/redimensionar"
            } else {
                "Travar mover/redimensionar"
            },
            topmost: "Alternar sempre no topo",
            login: "Entrar / configurar site",
            quit: "Sair",
            tooltip: "Floatly — Janela de site",
        },
    }
}

pub fn settings_title(language: &str) -> &'static str {
    match language {
        "en" => "Settings — Floatly",
        "es" => "Configuración — Floatly",
        _ => "Configurações — Floatly",
    }
}

pub fn login_title(language: &str) -> &'static str {
    match language {
        "en" => "Sign in",
        "es" => "Iniciar sesión",
        _ => "Entrar",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_labels_have_distinct_lock_texts_for_each_state_and_language() {
        for language in ["pt-BR", "en", "es"] {
            let locked = tray_labels(language, true);
            let unlocked = tray_labels(language, false);
            assert_ne!(locked.lock, unlocked.lock);
            assert!(!locked.lock.is_empty());
            assert!(!unlocked.lock.is_empty());
        }
        assert_eq!(
            tray_labels("pt-BR", true).lock,
            "Destravar mover/redimensionar"
        );
        assert_eq!(
            tray_labels("pt-BR", false).lock,
            "Travar mover/redimensionar"
        );
    }

    #[test]
    fn unknown_language_falls_back_to_pt_br() {
        assert_eq!(tray_labels("unknown", false), tray_labels("pt-BR", false));
        assert_eq!(settings_title("unknown"), settings_title("pt-BR"));
        assert_eq!(login_title("unknown"), login_title("pt-BR"));
    }
}
