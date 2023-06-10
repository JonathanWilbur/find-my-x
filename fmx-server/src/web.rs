use yew::prelude::*;
use crate::grpc::find_my_device::LocationSnapshot;
use std::rc::Rc;
use crate::utils::grpc_timestamp_to_chrono;

#[derive(Properties, PartialEq)]
pub struct LocationHistoryItemProps {
    pub snapshot: Rc<LocationSnapshot>,
    pub alternation: bool,
}

const UNSUPPLIED_FIELD: &str = "-";

#[function_component]
fn LocationHistoryItem (props: &LocationHistoryItemProps) -> Html {
    let update_time = props
        .snapshot
        .update_time
        .as_ref()
        .and_then(|t| grpc_timestamp_to_chrono(&t))
        .map(|t| t.to_rfc2822())
        .unwrap_or(String::from(UNSUPPLIED_FIELD));
    let (lat, long, elevation) = props
        .snapshot
        .location
        .as_ref()
        .and_then(|loc| Some((
            loc.degrees_latitude.to_string(),
            loc.degress_longitude.to_string(),
            loc.meters_elevation.to_string()
        )))
        .unwrap_or((String::from(UNSUPPLIED_FIELD), String::from(UNSUPPLIED_FIELD), String::from(UNSUPPLIED_FIELD)));
    let (speed, bearing) = props
        .snapshot
        .velocity
        .as_ref()
        .and_then(|vel| Some((
            vel.meters_per_second_speed.to_string(),
            vel.bearing.to_string(),
        )))
        .unwrap_or((String::from(UNSUPPLIED_FIELD), String::from(UNSUPPLIED_FIELD)));
    let next_update = props
        .snapshot
        .expected_next_update_time
        .as_ref()
        .and_then(|t| grpc_timestamp_to_chrono(&t))
        .map(|t| t.to_rfc2822())
        .unwrap_or(String::from(UNSUPPLIED_FIELD));
    let emergency = if props.snapshot.emergency { "emergency" } else { "safe" };
    let notes = if props.snapshot.notes.len() > 0 {
        props.snapshot.notes.clone()
    } else {
        String::from(UNSUPPLIED_FIELD)
    };

    let wifi = if props.snapshot.nearby_wifi_network.len() > 0 {
        String::from(UNSUPPLIED_FIELD)
    } else {
        String::from(UNSUPPLIED_FIELD)
    };

    let bluetooth = if props.snapshot.nearby_bluetooth_devices.len() > 0 {
        String::from(UNSUPPLIED_FIELD)
    } else {
        String::from(UNSUPPLIED_FIELD)
    };

    let url_cell = if lat.len() > 1 && long.len() > 1 {
        let url = format!("https://www.openstreetmap.org/?mlat={}&mlon={}", lat, long);
        html!{<a href={url}>{"Link"}</a>}
    } else {
        html!{<>{String::from(UNSUPPLIED_FIELD)}</>}
    };
    
    return html! {
        <tr class={classes!(["loc-item", emergency].as_ref())}>
            <td>{update_time}</td>
            <td>{lat}</td>
            <td>{long}</td>
            <td>{elevation}</td>
            <td>{speed}</td>
            <td>{bearing}</td>
            <td>{next_update}</td>
            <td>{wifi}</td>
            <td>{bluetooth}</td>
            <td>{notes}</td>
            <td>{url_cell}</td>
        </tr>
    };
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub locations: Vec<Rc<LocationSnapshot>>,
}

const LOCATIONS_STYLE: &str = r#"
body {
    background: black;
    color: white;
}
.loc-item > td {
    padding: 4px;
}
a {
    color: #F44336;
}
a:hover {
    color: #EF9A9A;
}
a:visited {
    color: #7B1FA2;
}
.emergency {
    background-color: rgba(1, 0, 0, 0.5);
}
@media screen and (prefers-color-scheme: light) {
    body {
        background-color: white;
        color: black;
    }
}
"#;

#[function_component]
pub fn LocationsPage (props: &Props) -> Html {
    let css = Html::from_html_unchecked(LOCATIONS_STYLE.into());
    html! {
        <html>
            <head>
                <title>{"Locations"}</title>
                <style>{css}</style>
            </head>
            <body>
                <h1>{"Locations"}</h1>
                <hr />
                <table>
                    <thead>
                        <tr>
                            <th>{"Time"}</th>
                            <th>{"Lat."}</th>
                            <th>{"Long."}</th>
                            <th>{"Elev."}</th>
                            <th>{"Speed (m/s)"}</th>
                            <th>{"Bearing"}</th>
                            <th>{"Next Update"}</th>
                            <th>{"Nearby Wifi"}</th>
                            <th>{"Nearby Bluetooth"}</th>
                            <th>{"Notes"}</th>
                            <th>{"OpenStreetMap"}</th>
                        </tr>
                    </thead>
                    <tbody>
                    {
                        props.locations.iter().enumerate().map(|(i, loc)| {
                            html!{
                                <LocationHistoryItem
                                    snapshot={loc.clone()}
                                    alternation={(i % 2) == 0}
                                    />
                            }
                        }).collect::<Html>()
                    }
                    </tbody>
                </table>
            </body>
        </html>
    }
}

