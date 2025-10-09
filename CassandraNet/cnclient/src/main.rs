/// Copyright (C) 2024-present WD Studios L.L.C. & CassandraNet Contributors.
use cncommon::profile::cnprofile::*;

fn main() {
    CnProfile::start_event_with_data("main", "main", 0);
    for i in 0..100 {
        CnProfile::start_event("add");
        let _ = cncommon::add(i, i);
        CnProfile::end_event();
    }
    CnProfile::end_event();
}
