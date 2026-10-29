# ADR 0005: Notificações, bandeja e segundo plano

- **Data:** 2026-10-28
- **Estado:** aceita

## Decisão

- **Agendador:** uma thread do `pauta-app` acorda a cada 20 segundos e chama `ReminderState::due` no `pauta-core`. Um intervalo fixo, em vez de dormir até o próximo lembrete, cobre suspensão, retomada e mudanças de relógio sem código especial: no primeiro ciclo depois de acordar, o que passou vira "perdido".
- **Perdidos (9.5):** um lembrete com mais de 10 minutos de atraso, dentro de uma janela de 72 horas, entra numa única notificação agrupada ("2 posts atrasados") e aparece como atrasado na tela Hoje.
- **Estado:** `.pauta/reminders.json` guarda os lembretes entregues e adiados. É descartável: se for apagado, os lembretes das últimas 72 horas reaparecem como perdidos.
- **Notificações nativas:** usamos o `notify-rust` diretamente, e não o plugin de notificação do Tauri, porque no Linux ele oferece botões de ação via D-Bus: Abrir, Copiar texto, Adiar 15 min e Adiar 1 h. Clicar no corpo abre o cartão.
- **Threads de espera:** cada notificação com botões mantém uma thread parada em `wait_for_action` até ser fechada. Para que notificações ignoradas não acumulem threads, há no máximo 16 esperando; acima disso a notificação aparece sem botões e sem thread. Todas pedem expiração de 30 minutos, que os servidores que a respeitam usam para fechá-las (o que encerra a espera). O que expirar sem ser visto continua na tela Hoje como atrasado.
- **Bandeja:** `tray-icon` do Tauri (AppIndicator, ativo por padrão no Ubuntu), com o próximo post, o próximo prazo e ações rápidas. Fechar a janela esconde em vez de sair.
- **Inicialização automática:** `tauri-plugin-autostart` com `--hidden`, ligada por padrão **somente em builds de release**, para que uma execução de desenvolvimento não se registre em `~/.config/autostart`.
- **Instância única:** `tauri-plugin-single-instance`. Rodar `pauta --capture` com o app aberto abre a captura rápida na instância existente.
- **Atalho global:** `Ctrl+Alt+P` pelo `tauri-plugin-global-shortcut`, que só funciona no X11. No Wayland (padrão do Ubuntu), o caminho é criar um atalho personalizado do GNOME para `pauta --capture`.

## Ressalva de ambiente

Iniciado de dentro do terminal do VS Code instalado via snap, o app (como qualquer app GTK) aborta com `Settings schema … does not contain a key named 'antialiasing'`, porque o snap aponta `GSETTINGS_SCHEMA_DIR` e `XDG_DATA_DIRS` para os próprios esquemas. Num terminal comum, ou pelo atalho do sistema, o problema não ocorre.
