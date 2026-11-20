# Default Pet

Kyouzai currently supports a single built-in default pet, configured through `pet.json`.

## Directory Structure

```text
assets/
└── default-pet/
    ├── pet.json
    └── spritesheet.png
```

## Configuration

The `pet.json` file defines the pet's metadata, spritesheet layout, movement speed, and animation states.

Example:

```json
{
  "id": "kyouzai",
  "name": "Kyouzai",
  "spritesheet": "spritesheet.png",
  "grid_columns": 4,
  "grid_rows": 2,
  "sprite_size": [128, 128],
  "walk_speed": 60.0,
  "states": {
    "idle": {
      "range": {
        "row": 0,
        "start": 0,
        "count": 4
      },
      "fps": 6.0
    },
    "walk": {
      "range": {
        "row": 1,
        "start": 0,
        "count": 4
      },
      "fps": 8.0
    }
  }
}
```

## Fields

| Field          | Description                                    |
| -------------- | ---------------------------------------------- |
| `id`           | Pet identifier.                                |
| `name`         | Pet display name.                              |
| `spritesheet`  | Spritesheet path, relative to `pet.json`.      |
| `grid_columns` | Number of spritesheet columns.                 |
| `grid_rows`    | Number of spritesheet rows.                    |
| `sprite_size`  | Frame dimensions in pixels: `[width, height]`. |
| `walk_speed`   | Pet movement speed.                            |
| `states`       | Animation definitions.                         |

## Animation States

Kyouzai currently supports the following built-in states:

| State     | Description                    |
| --------- | ------------------------------ |
| `idle`    | Default idle animation.        |
| `walk`    | Walking animation.             |
| `sleep`   | Sleeping animation.            |
| `grabbed` | Animation while being dragged. |
| `falling` | Falling animation.             |

### State Rules

- `idle` is required.
- Other states are optional.
- If a requested state is not defined, Kyouzai falls back to `idle`.
- State names must match the built-in animation keys.

## Animation Definition

Each animation defines its playback speed and frame sequence.

### Range

Use `range` for consecutive frames in the same row.

```json
{
  "range": {
    "row": 0,
    "start": 0,
    "count": 4
  },
  "fps": 6.0
}
```

### Frames

Use `frames` for an explicit sequence of frame coordinates.

```json
{
  "frames": [
    [0, 0],
    [1, 0],
    [2, 1],
    [3, 1]
  ],
  "fps": 8.0
}
```

Frame coordinates use `[column, row]`, starting from zero.

## Spritesheet Rules

- The spritesheet uses a uniform grid.
- Each frame must have the declared `sprite_size`.
- Image dimensions must match the declared grid.

```text
image_width  = grid_columns × sprite_width
image_height = grid_rows × sprite_height
```

- All referenced frame coordinates must be within the grid.
- Animation frame sequences must not be empty.
- `walk_speed` must be non-negative.
- Animation `fps` must be greater than zero.
