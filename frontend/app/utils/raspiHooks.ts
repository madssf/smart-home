export const useTriggerRefresh = async() => {
    return fetch("http://raspi-rust-api:8080/trigger_refresh")
        .then(res => {
            if (!res.ok) {
                console.warn('Failed to trigger refresh, response status:', res.status)
            }
        })
        .catch((e) =>
            console.warn('Error when triggering raspi refresh:', e)
        )
}
