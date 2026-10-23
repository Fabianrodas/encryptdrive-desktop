use encryptdrive_spike_common::{
    fixture::make_fixture,
    model::{DialogKind, EntryId, ExplorerModel, Page, SortColumn},
    theme::{SemanticTokens, ThemeMode},
};

#[test]
fn fixture_sizes_are_exact_and_ids_are_stable() {
    for size in [100, 1_000, 10_000] {
        let first = make_fixture(size);
        let second = make_fixture(size);
        assert_eq!(
            ExplorerModel::new(first.clone()).visible_ids().len(),
            size,
            "the root list has exactly {size} visible rows"
        );
        assert_eq!(first.entries(), second.entries());
        assert_eq!(
            first
                .entries()
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            second
                .entries()
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn selection_survives_sorting_by_stable_id_and_select_all_is_visible_only() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let chosen = model.visible_ids()[0];
    model.select_only(chosen);
    model.set_sort(SortColumn::Size, false);
    assert!(model.selected_ids().contains(&chosen));

    model.set_search("Synthetic document 000");
    let visible = model.visible_ids();
    model.select_all_visible();
    assert_eq!(model.selected_ids().len(), visible.len());
    assert!(model.selected_ids().iter().all(|id| visible.contains(id)));
}

#[test]
fn descending_name_sort_keeps_folders_before_files() {
    let mut model = ExplorerModel::new(make_fixture(100));
    model.set_sort(SortColumn::Name, false);
    let visible = model.visible_entries();
    let first_file = visible
        .iter()
        .position(|entry| entry.kind == encryptdrive_spike_common::EntryKind::File)
        .unwrap();
    assert!(
        visible[..first_file]
            .iter()
            .all(|entry| entry.kind == encryptdrive_spike_common::EntryKind::Folder)
    );
}

#[test]
fn keyboard_style_range_selection_uses_the_current_visible_order() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let visible = model.visible_ids();
    let first = visible[1];
    let last = visible[4];
    model.select_only(first);
    model.select_range(last);
    let expected: std::collections::BTreeSet<EntryId> = visible[1..=4].iter().copied().collect();
    let selected: std::collections::BTreeSet<EntryId> = model.selected_ids().into_iter().collect();
    assert_eq!(selected, expected);
}

#[test]
fn files_and_trash_navigation_preserve_the_files_location() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let folder = model
        .entries()
        .iter()
        .find(|entry| entry.name == "Projects")
        .unwrap()
        .id;
    model.navigate_into(folder);
    model.set_page(Page::Trash);
    model.set_page(Page::Files);
    assert_eq!(model.current_folder(), folder);
    assert_eq!(model.breadcrumb().last().unwrap().1, "Projects");
}

#[test]
fn folder_activation_navigates_and_file_activation_opens_export_warning() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let folder = model
        .entries()
        .iter()
        .find(|entry| entry.name == "Projects")
        .unwrap()
        .id;
    model.activate(folder);
    assert_eq!(model.current_folder(), folder);

    let nested = model
        .visible_entries()
        .into_iter()
        .find(|entry| entry.kind == encryptdrive_spike_common::EntryKind::Folder)
        .unwrap()
        .id;
    model.activate(nested);

    let file = model
        .visible_entries()
        .into_iter()
        .find(|entry| entry.kind == encryptdrive_spike_common::EntryKind::File)
        .unwrap()
        .id;
    model.activate(file);
    assert_eq!(
        model.dialog().map(|dialog| dialog.kind),
        Some(DialogKind::ExportWarning)
    );
}

#[test]
fn synthetic_name_validation_rejects_blank_and_case_insensitive_collisions() {
    let mut model = ExplorerModel::new(make_fixture(100));
    assert!(model.create_folder("   ").is_err());
    assert!(model.create_folder("budget.xlsx").is_ok());
    model.navigate_into(EntryId(5)); // Collision Lab has the controlled active collision.
    assert!(model.create_folder("budget.xlsx").is_err());
    assert!(model.create_folder("New folder").is_ok());
}

#[test]
fn operation_cancel_and_failure_have_distinct_terminal_states() {
    let mut model = ExplorerModel::new(make_fixture(100));
    model.start_operation("Synthetic import", 20, None);
    model.cancel_operation();
    assert!(model.operation().is_cancelled());

    model.start_operation("Synthetic export", 2, Some(1));
    model.advance_operation();
    assert!(model.operation().is_failed());
}

#[test]
fn delete_command_uses_trash_or_permanent_delete_for_the_current_page() {
    let mut model = ExplorerModel::new(make_fixture(100));
    assert_eq!(model.delete_dialog_kind(), DialogKind::DeleteToTrash);

    model.set_page(Page::Trash);
    assert_eq!(model.delete_dialog_kind(), DialogKind::PermanentDelete);
}

#[test]
fn windows_name_validation_rejects_device_names_trailing_dots_and_controls() {
    let mut model = ExplorerModel::new(make_fixture(100));
    for invalid_name in [
        "CON",
        "con.txt",
        "NUL.bin",
        "COM1.log",
        "COM¹",
        "LPT9",
        "LPT³.txt",
        "name.",
        "bad\u{001F}name",
    ] {
        assert!(
            model.create_folder(invalid_name).is_err(),
            "Windows-incompatible name should be rejected: {invalid_name:?}"
        );
    }
    assert!(model.create_folder("Résumé 東京").is_ok());
}

#[test]
fn ctrl_selection_keeps_stable_ids_when_the_visible_order_changes() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let visible = model.visible_ids();
    let first = visible[1];
    let second = visible[4];

    model.select_only(first);
    model.toggle_selection(second);
    model.set_sort(SortColumn::Size, false);

    let selected: std::collections::BTreeSet<EntryId> = model.selected_ids().into_iter().collect();
    assert_eq!(selected, [first, second].into_iter().collect());
    assert!(model.visible_ids().contains(&first));
    assert!(model.visible_ids().contains(&second));
}

#[test]
fn shift_range_uses_the_current_order_after_sorting() {
    let mut model = ExplorerModel::new(make_fixture(100));
    model.set_sort(SortColumn::Size, false);
    let visible = model.visible_ids();
    let anchor = visible[1];
    let target = visible[5];

    model.select_only(anchor);
    model.select_range(target);

    let expected: std::collections::BTreeSet<EntryId> = visible[1..=5].iter().copied().collect();
    let selected: std::collections::BTreeSet<EntryId> = model.selected_ids().into_iter().collect();
    assert_eq!(selected, expected);
    assert_eq!(model.focused_id(), Some(target));
}

#[test]
fn unicode_and_long_fixture_names_are_preserved_exactly() {
    let fixture = make_fixture(100);
    assert!(
        fixture
            .entries()
            .iter()
            .any(|entry| entry.name == "résumé — 東京.txt")
    );
    assert!(fixture.entries().iter().any(|entry| {
        entry.name == "A deliberately long synthetic document name for truncation and tooltip checks 2026.txt"
    }));
}

#[test]
fn clearing_search_restores_results_without_resurrecting_filtered_selection() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let folder = model
        .visible_entries()
        .into_iter()
        .find(|entry| entry.name == "Projects")
        .unwrap()
        .id;
    model.select_only(folder);

    model.set_search("Synthetic document 00007");
    assert!(model.selected_ids().is_empty());
    assert_eq!(model.visible_entries().len(), 1);
    assert_eq!(
        model.visible_entries()[0].name,
        "Synthetic document 00007.png"
    );

    model.set_search("");
    assert_eq!(model.visible_ids().len(), 100);
    assert!(model.selected_ids().is_empty());
}

#[test]
fn cancelled_queued_import_does_not_change_synthetic_entries() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let initial_count = model.entries().len();
    model.start_operation("Synthetic import", 10, None);
    model.queue_after_success(encryptdrive_spike_common::SyntheticAction::Import {
        parent: EntryId(0),
        name: "Cancelled import.txt".to_owned(),
    });

    model.cancel_operation();
    model.finish_operation();

    assert_eq!(model.entries().len(), initial_count);
    assert!(
        model
            .entries()
            .iter()
            .all(|entry| entry.name != "Cancelled import.txt")
    );
}

#[test]
fn trash_restore_preserves_id_clears_deleted_time_and_resolves_name_collision() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let deleted = model
        .entries()
        .into_iter()
        .find(|entry| entry.deleted_at.is_some())
        .unwrap()
        .clone();

    model.set_page(Page::Trash);
    assert!(model.visible_ids().contains(&deleted.id));
    assert_eq!(model.restore(&[deleted.id]), vec![deleted.id]);

    let restored = model.entry(deleted.id).unwrap();
    assert_eq!(restored.id, deleted.id);
    assert_eq!(restored.name, "Budget (restored 2).xlsx");
    assert_eq!(restored.deleted_at, None);
    assert_eq!(model.page(), Page::Files);
}

#[test]
fn rename_dialog_prefills_target_and_invalid_name_does_not_mutate_the_entry() {
    let mut model = ExplorerModel::new(make_fixture(100));
    let target = model
        .visible_entries()
        .into_iter()
        .find(|entry| entry.name == "image sample.png")
        .unwrap()
        .id;

    model.open_dialog(DialogKind::Rename, Some(target));
    let dialog = model.dialog().unwrap();
    assert_eq!(dialog.name, "image sample.png");
    assert_eq!(dialog.target, Some(target));
    assert_eq!(dialog.error, None);

    assert!(model.rename(target, "invalid/name.png").is_err());
    assert_eq!(model.entry(target).unwrap().name, "image sample.png");
    model.close_dialog();
    assert!(model.dialog().is_none());
}

#[test]
fn semantic_text_selection_and_focus_tokens_keep_contrast_in_light_and_dark() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let tokens = SemanticTokens::for_mode(mode);
        assert!(contrast_ratio(tokens.primary_text, tokens.surface) >= 4.5);
        assert!(contrast_ratio(tokens.secondary_text, tokens.surface) >= 4.5);
        assert!(contrast_ratio(tokens.selection_text, tokens.selection) >= 4.5);
        assert!(contrast_ratio(tokens.focus, tokens.surface) >= 3.0);
        assert!(contrast_ratio(tokens.focus, tokens.selection) >= 3.0);
    }
}

fn contrast_ratio(foreground: u32, background: u32) -> f64 {
    fn luminance(color: u32) -> f64 {
        fn linear(channel: u8) -> f64 {
            let value = f64::from(channel) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }

        0.2126 * linear((color >> 16) as u8)
            + 0.7152 * linear((color >> 8) as u8)
            + 0.0722 * linear(color as u8)
    }

    let foreground = luminance(foreground);
    let background = luminance(background);
    (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
}
