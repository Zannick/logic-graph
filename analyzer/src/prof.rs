// Note this whole module cannot be used with msvc.

use axum::{extract::Query, http::StatusCode, response::IntoResponse};
use pprof::protos::Message;
use std::{thread::sleep, time::Duration};
use tokio::{net::TcpListener, runtime::Runtime};

#[cfg(feature = "jemalloc")]
mod jemalloc {
    use axum::{http::StatusCode, response::IntoResponse};

    pub async fn handle_get_heap() -> Result<impl IntoResponse, (StatusCode, String)> {
        let mut prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().unwrap().lock().await;
        require_profiling_activated(&prof_ctl)?;
        let pprof = prof_ctl
            .dump_pprof()
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        Ok(pprof)
    }

    /// Checks whether jemalloc profiling is activated an returns an error response if not.
    fn require_profiling_activated(
        prof_ctl: &jemalloc_pprof::JemallocProfCtl,
    ) -> Result<(), (StatusCode, String)> {
        if prof_ctl.activated() {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "heap profiling not activated".into()))
        }
    }
}

#[derive(serde::Deserialize)]
struct ProfileArgs {
    dur: Option<u64>,
}

async fn cpu_profile(
    Query(q): Query<ProfileArgs>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sleep(Duration::from_secs(q.dur.unwrap_or(30)));
    match guard.report().build() {
        Ok(report) => {
            let profile = report.pprof().unwrap();
            let mut content = Vec::new();
            profile.encode(&mut content).unwrap();
            Ok(content)
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub(crate) fn start_profile_handlers(rt: &Runtime) {
    rt.spawn(async {
        let app = axum::Router::new().route("/debug/pprof", axum::routing::get(cpu_profile));
        #[cfg(feature = "jemalloc")]
        let app = app.route(
            "/debug/pprof/heap",
            axum::routing::get(jemalloc::handle_get_heap),
        );

        // run our app with hyper, listening globally on port 3000
        let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}
