use dioxus::prelude::*;

/// Wraps the Dioxus router [`Outlet`] in a keyed container with the app global class
/// `route-transition-outlet` (`@keyframes route-content-fade-in` in `services/app/assets/styles.scss`)
/// so the active route remounts with a short enter fade.
#[component]
pub fn RouteTransitionOutlet<R>() -> Element
where
    R: Routable + Clone + PartialEq + 'static,
{
    let router_ctx = router();
    // Key on the path only: query-string-only changes (e.g. `?tab=`) should not
    // tear down and remount the whole route subtree.
    let route_key = use_memo(move || {
        let full = router_ctx.current::<R>().to_string();
        match full.split_once('?') {
            Some((path, _)) => path.to_string(),
            None => full,
        }
    });

    rsx! {
        div {
            key: "{route_key()}",
            class: "route-transition-outlet",
            Outlet::<R> {}
        }
    }
}
