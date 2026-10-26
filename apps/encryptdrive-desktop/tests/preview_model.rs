#![cfg(feature = "design-preview")]

use encryptdrive_desktop::preview::{
    DatasetSize, Destination, EntryId, ExplorerState, Modifier, NameValidationError, PreviewModel,
    SortKey,
};

#[test]
fn fixture_has_exact_visible_list_sizes_and_stable_ids() {
    for (size, expected) in [
        (DatasetSize::OneHundred, 100),
        (DatasetSize::OneThousand, 1_000),
        (DatasetSize::TenThousand, 10_000),
    ] {
        let first = PreviewModel::fixture(size);
        let second = PreviewModel::fixture(size);
        let first_ids = first.list_ids(Destination::Files, None);

        assert_eq!(first_ids.len(), expected);
        assert_eq!(first_ids, second.list_ids(Destination::Files, None));
    }
}

#[test]
fn fixture_contains_nested_and_empty_folders_with_unicode_and_timestamps() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let root = model.list_ids(Destination::Files, None);
    let empty = root
        .iter()
        .map(|id| model.entry(*id).expect("fixture ID exists"))
        .find(|entry| entry.name == "Empty folder")
        .expect("empty folder fixture exists");
    let documents = root
        .iter()
        .map(|id| model.entry(*id).expect("fixture ID exists"))
        .find(|entry| entry.name == "Documents")
        .expect("nested folder fixture exists");

    assert!(
        model
            .list_ids(Destination::Files, Some(empty.id))
            .is_empty()
    );
    assert!(
        !model
            .list_ids(Destination::Files, Some(documents.id))
            .is_empty()
    );
    assert!(model.entries().any(|entry| entry.name.contains("日本語")));
    assert!(
        model
            .entries()
            .any(|entry| entry.name.chars().count() > 100)
    );
    assert!(model.entries().all(|entry| !entry.created_at.is_empty()));
    assert!(model.entries().all(|entry| !entry.modified_at.is_empty()));
    assert!(model.list_ids(Destination::Trash, None).iter().all(|id| {
        model
            .entry(*id)
            .and_then(|item| item.deleted_at.as_ref())
            .is_some()
    }));
}

#[test]
fn filter_changes_clear_selection_while_sorting_and_unchanged_queries_preserve_it() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    let initial = state.visible_ids(&model);
    let selected = initial[0];
    state.select(selected, &initial, Modifier::Plain);

    state.set_sort(SortKey::Size);
    assert!(state.selected_ids().contains(&selected));

    state.set_query(" 日本語 ");

    assert_eq!(state.visible_ids(&model).len(), 2);
    assert!(!state.visible_ids(&model).contains(&selected));
    assert!(state.selected_ids().is_empty());

    let visible = state.visible_ids(&model);
    state.select(visible[0], &visible, Modifier::Plain);
    let filtered_selection = state.selected_ids();
    state.set_query("日本語");
    assert_eq!(state.selected_ids(), filtered_selection);

    state.set_query("");
    assert!(state.selected_ids().is_empty());
}

#[test]
fn control_click_toggles_only_the_requested_stable_id() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    let visible = state.visible_ids(&model);
    let first = visible[0];
    let second = visible[1];

    state.select(first, &visible, Modifier::Plain);
    state.select(second, &visible, Modifier::Control);
    assert_eq!(state.selected_ids().len(), 2);
    state.select(first, &visible, Modifier::Control);
    assert_eq!(state.selected_ids().as_slice(), &[second]);
}

#[test]
fn plain_arrow_focus_moves_the_selection_to_the_focused_row() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    let visible = state.visible_ids(&model);
    let first = visible[0];

    state.move_focus(1, &visible, Some(Modifier::Plain));

    assert_eq!(state.focused_id(), Some(first));
    assert_eq!(state.selected_ids(), vec![first]);
}

#[test]
fn shift_arrow_extends_the_selection_from_the_anchor() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    let visible = state.visible_ids(&model);

    state.select(visible[2], &visible, Modifier::Plain);
    state.move_focus(1, &visible, Some(Modifier::Shift));

    assert_eq!(state.focused_id(), Some(visible[3]));
    assert_eq!(state.selected_ids().len(), 2);
    assert!(state.selected_ids().contains(&visible[2]));
    assert!(state.selected_ids().contains(&visible[3]));
}

#[test]
fn direct_keyboard_navigation_clamps_home_end_and_page_targets() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    let visible = state.visible_ids(&model);

    state.move_focus_to_index(usize::MAX, &visible, Some(Modifier::Plain));
    assert_eq!(state.focused_id(), visible.last().copied());
    assert_eq!(state.selected_ids(), vec![*visible.last().unwrap()]);

    state.move_focus_to_index(0, &visible, Some(Modifier::Plain));
    assert_eq!(state.focused_id(), Some(visible[0]));
    assert_eq!(state.selected_ids(), vec![visible[0]]);
}

#[test]
fn space_toggle_uses_the_focused_stable_id_and_preserves_other_selection() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    let visible = state.visible_ids(&model);

    state.select(visible[1], &visible, Modifier::Plain);
    state.select(visible[3], &visible, Modifier::Control);
    state.toggle_focused_selection(&visible);
    assert_eq!(state.focused_id(), Some(visible[3]));
    assert_eq!(state.selected_ids(), vec![visible[1]]);

    state.toggle_focused_selection(&visible);
    let selected = state.selected_ids();
    assert_eq!(selected.len(), 2);
    assert!(selected.contains(&visible[1]));
    assert!(selected.contains(&visible[3]));
}

#[test]
fn shift_click_selects_visible_range_from_anchor() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    let visible = state.visible_ids(&model);

    state.select(visible[2], &visible, Modifier::Plain);
    state.select(visible[6], &visible, Modifier::Shift);

    assert_eq!(state.selected_ids().len(), 5);
    for id in &visible[2..=6] {
        assert!(state.selected_ids().contains(id));
    }
}

#[test]
fn select_all_targets_only_current_visible_results() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    state.set_query("résumé");
    let visible = state.visible_ids(&model);

    state.select_all_visible(&visible);

    let mut selected = state.selected_ids();
    let mut expected = visible.to_vec();
    selected.sort();
    expected.sort();
    assert_eq!(selected, expected);
}

#[test]
fn sibling_name_collisions_are_rejected_without_mutating_fixture() {
    let mut model = PreviewModel::fixture(DatasetSize::OneHundred);
    let root = model.list_ids(Destination::Files, None);
    let file = root
        .iter()
        .find(|id| {
            model
                .entry(**id)
                .is_some_and(|entry| entry.name == "Annual report.txt")
        })
        .copied()
        .expect("report fixture exists");

    assert!(model.rename(file, "Documents").is_err());
    assert_eq!(
        model.entry(file).expect("same ID still exists").name,
        "Annual report.txt"
    );
}

#[test]
fn synthetic_trash_restore_and_permanent_delete_preserve_expected_ids() {
    let mut model = PreviewModel::fixture(DatasetSize::OneHundred);
    let root_id = model
        .list_ids(Destination::Files, None)
        .into_iter()
        .find(|id| {
            model
                .entry(*id)
                .is_some_and(|entry| entry.name == "Annual report.txt")
        })
        .expect("report fixture exists");

    model.move_to_trash(&[root_id], "2026-10-19 09:00");
    assert!(!model.list_ids(Destination::Files, None).contains(&root_id));
    assert!(model.list_ids(Destination::Trash, None).contains(&root_id));
    model.restore(&[root_id]);
    assert!(model.list_ids(Destination::Files, None).contains(&root_id));
    model.move_to_trash(&[root_id], "2026-10-19 09:00");
    model.permanently_delete(&[root_id]);
    assert!(model.entry(root_id).is_none());
}

#[test]
fn restoring_a_nested_trash_entry_restores_its_deleted_ancestors() {
    let mut model = PreviewModel::fixture(DatasetSize::OneHundred);
    let documents = EntryId(1);
    let project = EntryId(101);
    let note = EntryId(102);
    model.move_to_trash(&[documents], "2026-10-19 10:00");

    model.restore(&[note]);

    assert!(
        model
            .list_ids(Destination::Files, None)
            .contains(&documents)
    );
    assert!(
        model
            .list_ids(Destination::Files, Some(documents))
            .contains(&project)
    );
    assert!(
        model
            .list_ids(Destination::Files, Some(documents))
            .contains(&note)
    );
    let trash = model.list_ids(Destination::Trash, None);
    assert!(!trash.contains(&documents));
    assert!(!trash.contains(&project));
    assert!(!trash.contains(&note));
}

#[test]
fn generated_entry_ids_are_not_reindexed_by_filter_or_sort() {
    let model = PreviewModel::fixture(DatasetSize::OneThousand);
    let tracked = EntryId(1_050);
    let mut state = ExplorerState::new();
    assert!(model.entry(tracked).is_some());
    state.set_query("project");
    state.set_sort(SortKey::ModifiedSample);
    assert_eq!(model.entry(tracked).map(|entry| entry.id), Some(tracked));
}

#[test]
fn folder_creation_trims_names_and_rejects_empty_invalid_or_duplicate_names() {
    let mut model = PreviewModel::fixture(DatasetSize::OneHundred);
    let created = model
        .create_folder(None, "  Work files  ", "2026-10-19 09:00")
        .expect("valid sample folder can be created");

    assert_eq!(
        model.entry(created).expect("new ID exists").name,
        "Work files"
    );
    assert_eq!(
        model.create_folder(None, "   ", "2026-10-19 09:00"),
        Err(NameValidationError::Empty)
    );
    assert_eq!(
        model.create_folder(None, "bad/name", "2026-10-19 09:00"),
        Err(NameValidationError::InvalidCharacters)
    );
    assert_eq!(
        model.create_folder(None, "work FILES", "2026-10-19 09:00"),
        Err(NameValidationError::NameAlreadyExists)
    );
}

#[test]
fn navigation_back_restores_parent_and_clears_stale_selection() {
    let model = PreviewModel::fixture(DatasetSize::OneHundred);
    let mut state = ExplorerState::new();
    let root = state.visible_ids(&model);
    let documents = root
        .iter()
        .find(|id| {
            model
                .entry(**id)
                .is_some_and(|entry| entry.name == "Documents")
        })
        .copied()
        .expect("Documents fixture exists");

    state.select(documents, &root, Modifier::Plain);
    state.navigate_to(Some(documents));
    assert_eq!(state.current_folder(), Some(documents));
    assert!(state.selected_ids().is_empty());
    assert!(state.navigate_back());
    assert_eq!(state.current_folder(), None);
    assert_eq!(state.visible_ids(&model).len(), 100);
}

#[test]
fn name_search_is_transient_and_sorting_does_not_reassign_ids() {
    let model = PreviewModel::fixture(DatasetSize::OneThousand);
    let mut state = ExplorerState::new();
    state.set_query("日本語");
    let matches = state.visible_ids(&model);

    assert!(!matches.is_empty());
    assert!(matches.iter().all(|id| model.entry(*id).is_some()));
    let selected = matches[0];
    state.select(selected, &matches, Modifier::Plain);
    state.set_sort(SortKey::ModifiedSample);
    assert!(state.selected_ids().contains(&selected));
    state.set_query("");
    assert!(state.query().is_empty());
}

#[test]
fn sorted_projection_is_cached_until_the_query_or_model_changes() {
    let mut model = PreviewModel::fixture(DatasetSize::TenThousand);
    let mut state = ExplorerState::new();
    let first = state.visible_ids(&model);
    let same = state.visible_ids(&model);
    assert!(std::sync::Arc::ptr_eq(&first, &same));

    let created = model
        .create_folder(None, "Cache invalidation sample", "2026-10-19 10:00")
        .expect("synthetic folder is created");
    let updated = state.visible_ids(&model);
    assert_eq!(updated.len(), first.len() + 1);
    assert!(updated.contains(&created));
}

mod recursive_operation_scope {
    use std::collections::BTreeSet;

    use encryptdrive_desktop::preview::{
        DatasetSize, Destination, EntryId, PlanError, PreviewModel, RecursiveOperation,
    };

    const TRASH: RecursiveOperation = RecursiveOperation::MoveToTrash;
    const DELETE: RecursiveOperation = RecursiveOperation::PermanentDelete;

    fn ids(values: &[u64]) -> BTreeSet<EntryId> {
        values.iter().copied().map(EntryId).collect()
    }

    fn model() -> PreviewModel {
        PreviewModel::fixture(DatasetSize::OneHundred)
    }

    #[test]
    fn one_selected_file_affects_only_itself() {
        let plan = model().plan(TRASH, &[EntryId(4)]);
        assert_eq!(plan.affected(), &ids(&[4]));
        assert_eq!(plan.included_count(), 0);
    }

    #[test]
    fn one_empty_folder_affects_only_itself() {
        let plan = model().plan(TRASH, &[EntryId(3)]);
        assert_eq!(plan.affected(), &ids(&[3]));
    }

    #[test]
    fn a_folder_with_children_includes_them() {
        let plan = model().plan(TRASH, &[EntryId(2)]);
        assert_eq!(plan.affected(), &ids(&[2, 201, 202]));
        assert_eq!((plan.selected_count(), plan.included_count()), (1, 2));
    }

    #[test]
    fn nested_folders_are_expanded_to_every_level() {
        let mut model = model();
        // Documents › Projects › A › B › C, plus the fixture's existing children.
        let a = model
            .create_folder(Some(EntryId(101)), "A", "2026-10-19 09:00")
            .unwrap();
        let b = model
            .create_folder(Some(a), "B", "2026-10-19 09:00")
            .unwrap();
        let c = model
            .create_folder(Some(b), "C", "2026-10-19 09:00")
            .unwrap();
        let plan = model.plan(TRASH, &[EntryId(1)]);
        let mut expected = ids(&[1, 101, 102, 103]);
        expected.extend([a, b, c]);
        assert_eq!(plan.affected(), &expected);
    }

    #[test]
    fn two_independent_folders_add_up_without_overlap() {
        let plan = model().plan(TRASH, &[EntryId(1), EntryId(2)]);
        assert_eq!(plan.affected(), &ids(&[1, 101, 102, 103, 2, 201, 202]));
        assert_eq!(plan.affected_count(), 7);
    }

    #[test]
    fn a_parent_and_its_descendant_selected_together_are_counted_once() {
        let plan = model().plan(TRASH, &[EntryId(1), EntryId(101), EntryId(103)]);
        assert_eq!(plan.affected(), &ids(&[1, 101, 102, 103]));
        assert_eq!(plan.selected_count(), 3);
        assert_eq!(
            plan.included_count(),
            1,
            "only Shared notes was not selected"
        );
    }

    #[test]
    fn duplicate_and_unknown_ids_are_ignored_safely() {
        let plan = model().plan(
            TRASH,
            &[EntryId(4), EntryId(4), EntryId(424_242), EntryId(4)],
        );
        assert_eq!(plan.selected(), &ids(&[4]));
        assert_eq!(plan.affected(), &ids(&[4]));
    }

    #[test]
    fn ten_thousand_selected_roots_affect_exactly_10_005_entries() {
        let model = PreviewModel::fixture(DatasetSize::TenThousand);
        let roots = model.list_ids(Destination::Files, None);
        assert_eq!(roots.len(), 10_000);
        let started = std::time::Instant::now();
        let plan = model.plan(TRASH, &roots);
        let elapsed = started.elapsed();
        let mut expected: BTreeSet<EntryId> = roots.iter().copied().collect();
        expected.extend([101, 102, 103, 201, 202].map(EntryId));
        assert_eq!(plan.affected(), &expected);
        assert_eq!(
            (plan.selected_count(), plan.affected_count()),
            (10_000, 10_005)
        );
        assert!(elapsed.as_secs_f64() < 1.0, "planning took {elapsed:?}");
    }

    #[test]
    fn executing_a_plan_changes_exactly_its_affected_set() {
        let mut model = PreviewModel::fixture(DatasetSize::TenThousand);
        let roots = model.list_ids(Destination::Files, None);
        let plan = model.plan(TRASH, &roots);
        assert_eq!(model.execute_plan(&plan, "2026-10-19 10:00"), Ok(10_005));
        let trashed: BTreeSet<EntryId> = model
            .entries()
            .filter(|entry| entry.deleted_at.as_deref() == Some("2026-10-19 10:00"))
            .map(|entry| entry.id)
            .collect();
        assert_eq!(&trashed, plan.affected());
    }

    #[test]
    fn permanent_deletion_of_a_folder_removes_its_descendants_only() {
        let mut model = model();
        model.move_to_trash(&[EntryId(1)], "2026-10-19 09:00");
        let plan = model.plan(DELETE, &[EntryId(1)]);
        assert_eq!(plan.affected(), &ids(&[1, 101, 102, 103]));
        assert_eq!(model.execute_plan(&plan, ""), Ok(4));
        for id in [1, 101, 102, 103] {
            assert!(model.entry(EntryId(id)).is_none());
        }
        assert!(
            model.entry(EntryId(90_000)).is_some(),
            "other Trash items remain"
        );
        assert!(model.entry(EntryId(2)).is_some(), "active items remain");
    }

    #[test]
    fn a_plan_made_before_a_change_is_refused_and_changes_nothing() {
        let mut model = model();
        let plan = model.plan(TRASH, &[EntryId(1)]);
        model.rename(EntryId(4), "Renamed.txt").unwrap();
        let revision = model.revision();
        assert_eq!(
            model.execute_plan(&plan, "2026-10-19 10:00"),
            Err(PlanError::Stale)
        );
        assert_eq!(model.revision(), revision);
        assert!(model.entry(EntryId(1)).unwrap().deleted_at.is_none());
    }

    #[test]
    fn destinations_are_kept_apart() {
        let mut model = model();
        // Shared notes goes to Trash on its own first.
        model.move_to_trash(&[EntryId(102)], "2026-10-19 08:00");
        // Moving Documents later neither re-trashes nor recounts the item already there.
        let plan = model.plan(TRASH, &[EntryId(1)]);
        assert_eq!(plan.affected(), &ids(&[1, 101, 103]));
        model.execute_plan(&plan, "2026-10-19 09:00").unwrap();
        assert_eq!(
            model.entry(EntryId(102)).unwrap().deleted_at.as_deref(),
            Some("2026-10-19 08:00")
        );
        // Trash IDs are not in Files scope, and active IDs are not in Trash scope.
        assert!(model.plan(TRASH, &[EntryId(90_000)]).affected().is_empty());
        assert!(model.plan(DELETE, &[EntryId(4)]).affected().is_empty());
    }
}
