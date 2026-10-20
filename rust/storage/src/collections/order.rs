use super::*;

pub(super) fn move_adjacent(
    db: &Connection,
    expected: &CollectionRevision,
    direction: &CollectionDirection,
) -> Result<()> {
    let delta = match direction {
        CollectionDirection::Earlier => -1,
        CollectionDirection::Later => 1,
    };
    let revision = revision_after(Some(&expected.updated_at));
    let alternate = revision_after(Some(&revision));
    // Rank the complete pin group so legacy ties and page boundaries cannot lose a move.
    db.execute(
        "WITH ordered AS MATERIALIZED (
            SELECT id,sort_order,ROW_NUMBER() OVER (ORDER BY sort_order,name COLLATE NOCASE,id)-1 AS position
            FROM collections WHERE is_pinned=(SELECT is_pinned FROM collections WHERE id=?1)
        ), origin AS (SELECT position FROM ordered WHERE id=?1),
        moved AS (
            SELECT id,CASE position
                WHEN (SELECT position FROM origin) THEN position+?2
                WHEN (SELECT position+?2 FROM origin) THEN position-?2
                ELSE position END AS destination FROM ordered
        )
        UPDATE collections SET
            sort_order=(SELECT destination FROM moved WHERE moved.id=collections.id),
            updated_at=CASE WHEN updated_at=?3 THEN ?4 ELSE ?3 END
        WHERE id IN (SELECT id FROM moved)
            AND EXISTS(SELECT 1 FROM ordered WHERE position=(SELECT position+?2 FROM origin))
            AND (sort_order<>(SELECT destination FROM moved WHERE moved.id=collections.id)
                OR id IN(SELECT id FROM ordered WHERE position IN
                    ((SELECT position FROM origin),(SELECT position+?2 FROM origin))))",
        params![expected.id,delta,revision,alternate],
    ).map_err(storage)?;
    Ok(())
}
