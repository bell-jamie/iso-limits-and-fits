pub struct ChangelogEntry {
    pub version: &'static str,
    pub notes: &'static [&'static str],
}

pub const ENTRIES: &[ChangelogEntry] = &[
    ChangelogEntry {
        version: "0.8.1",
        notes: &[
            "Double component allocation fixed",
            "Temporarily removed ID / OD",
        ],
    },
    ChangelogEntry {
        version: "0.8.0",
        notes: &[
            "Material and component library",
            "Thermal fit display",
            "Limits and fits display",
            "Refined styling",
        ],
    },
    ChangelogEntry {
        version: "0.7.0",
        notes: &["Simple mode and advanced mode", "Zoom feature tweaked"],
    },
    ChangelogEntry {
        version: "0.6.4",
        notes: &["Corrected logic for P to ZC deviation deltas"],
    },
    ChangelogEntry {
        version: "0.6.3",
        notes: &["Quickfix to lookup table"],
    },
    ChangelogEntry {
        version: "0.6.2",
        notes: &[
            "Temperature sync",
            "Separate temperature output",
            "UI tweaks",
        ],
    },
    ChangelogEntry {
        version: "0.6.1",
        notes: &["Zoom feature"],
    },
    ChangelogEntry {
        version: "0.6.0",
        notes: &["Thermal fit analysis", "General UI tweaks and new symbols"],
    },
    ChangelogEntry {
        version: "0.5.2",
        notes: &[
            "Fixed manual limits not working",
            "Tooltips added",
            "Header bar tweaked",
        ],
    },
    ChangelogEntry {
        version: "0.5.1",
        notes: &["Minor UI change for fits"],
    },
    ChangelogEntry {
        version: "0.5.0",
        notes: &["Full ISO limits and fits tables", "Debug mode"],
    },
];
