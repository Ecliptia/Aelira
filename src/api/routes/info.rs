use warp::Filter;
use crate::aelira::AeliraRef;
use serde_json::json;
use sysinfo::System;

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    warp::path("v4")
        .and(warp::path("info"))
        .and(warp::get())
        .and(with_aelira)
        .map(|aelira: AeliraRef| {
            aelira.stats.increment_api_request("/v4/info");
            let version_str = aelira.version.clone();
            let parts: Vec<&str> = version_str.split('.').collect();
            let major = parts.get(0).unwrap_or(&"0").parse().unwrap_or(0);
            let minor = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
            let patch = parts.get(2).unwrap_or(&"0").parse().unwrap_or(0);

            let os = System::name().unwrap_or_else(|| "unknown".to_string());
            let _kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());

            let response = json!({
                "version": {
                    "semver": version_str,
                    "major": major,
                    "minor": minor,
                    "patch": patch,
                    "prerelease": null,
                    "build": null
                },
                "buildTime": -1,
                "git": {
                    "branch": "unknown",
                    "commit": "unknown",
                    "commitTime": -1
                },
                "rust": {
                    "version": "1.84.0",
                    "os": os,
                    "arch": std::env::consts::ARCH
                },
                "voice": {
                    "name": "aelira-voice",
                    "version": "1.0.0"
                },
                "sourceManagers": ["local"],
                "filters": [],
                "plugins": []
            });

            warp::reply::json(&response)
        })
}
