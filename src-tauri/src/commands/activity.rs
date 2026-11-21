use crate::state::{AppState, lock};
use crate::time::now_unix;
use serde::Serialize;
use tauri::State;

const WEEK_SECONDS: i64 = 7 * 86_400;
const MIN_SESSION_SECONDS: u64 = 120;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSpan {
    game_id: String,
    started_at: i64,
    ended_at: i64,
}

#[tauri::command]
pub fn recent_sessions(state: State<'_, AppState>) -> Vec<SessionSpan> {
    let since = now_unix() - WEEK_SECONDS;
    let mut spans: Vec<SessionSpan> = lock(&state.playtime)
        .intervals_since(since)
        .into_iter()
        .map(|(game_id, started_at, ended_at)| SessionSpan {
            game_id,
            started_at,
            ended_at,
        })
        .collect();
    spans.extend(
        lock(&state.journal)
            .iter()
            .filter(|session| session.ended_at >= since && session.seconds >= MIN_SESSION_SECONDS)
            .filter_map(|session| {
                let app_id = session.desktop_stem.strip_prefix("steam-app")?;
                Some(SessionSpan {
                    game_id: format!("steam:{app_id}"),
                    started_at: session.ended_at - session.seconds as i64,
                    ended_at: session.ended_at,
                })
            }),
    );
    spans.sort_by_key(|span| span.started_at);
    spans
}
