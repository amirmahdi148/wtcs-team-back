# WTCS Team Back

Backend API for members profile data using Rust (`actix-web`) + PostgreSQL (`sqlx`).

## Current Data Model

### Core tables
- `members`: account/profile base info.
- `badges`: shared badge catalog.
- `skills`: shared skill catalog.
- `about`: shared about-item catalog.
- `quotes`: per-member quotes.

### Polymorphic metadata redesign
To support one member linking to different metadata types safely, the backend now uses:
- `meta_items(id, meta_type, source_id)`
- `members_meta(owner, meta_item_id, meta_type, level)`

Linked relations:
- `badges.meta_item_id -> meta_items.id`
- `skills.meta_item_id -> meta_items.id`
- `about.meta_item_id -> meta_items.id`
- `members_meta.meta_item_id -> meta_items.id`
- `members_meta(meta_item_id, meta_type) -> meta_items(id, meta_type)`

This guarantees that a member link points to the correct typed resource.

## Skill Level Rule
`members_meta.level` is **skill-only**.

DB constraint:
- If `meta_type = 'skill'`: `level` is required and must be `0..100`.
- If `meta_type IN ('badge', 'about')`: `level` must be `NULL`.

So badges/about cannot accidentally store progress values.

## How Member Detail Is Built
`get_member_detail` loads:
- member base data
- badges via `members_meta.meta_item_id`
- skills via `members_meta.meta_item_id` (including `level`)
- about items via `members_meta.meta_item_id`
- quotes by member id

## Tutorial

### 1) Add a shared catalog item
Add to the source table (`badges`, `skills`, or `about`).

### 2) Link it to a member
Link by using table `meta_item_id` in `members_meta`.

#### Link badge
```sql
INSERT INTO members_meta (owner, meta_item_id, meta_type)
SELECT $member_id, b.meta_item_id, 'badge'
FROM badges b
WHERE b.id = $badge_id;
```

#### Link skill with level
```sql
INSERT INTO members_meta (owner, meta_item_id, meta_type, level)
SELECT $member_id, s.meta_item_id, 'skill', $level
FROM skills s
WHERE s.id = $skill_id;
```

#### Link about
```sql
INSERT INTO members_meta (owner, meta_item_id, meta_type)
SELECT $member_id, a.meta_item_id, 'about'
FROM about a
WHERE a.id = $about_id;
```

### 3) Update skill level
```sql
UPDATE members_meta
SET level = $new_level
WHERE owner = $member_id
  AND meta_type = 'skill'
  AND meta_item_id = (
      SELECT s.meta_item_id
      FROM skills s
      WHERE s.id = $skill_id
  );
```

## Validation Queries

```sql
-- Check all member links
SELECT mm.id, mm.owner, mm.meta_type, mm.meta_item_id, mm.level,
       mi.source_id
FROM members_meta mm
JOIN meta_items mi ON mi.id = mm.meta_item_id
ORDER BY mm.id;

-- Count catalog items by type
SELECT meta_type, count(*)
FROM meta_items
GROUP BY meta_type
ORDER BY meta_type;
```
