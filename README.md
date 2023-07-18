# launchpad-ji
An MPE midi effect for the launchpad to play in just intonated tonality.

Tested in:
- ableton live ❌
- renoise ❌
- bitwig ✔️

## adapting for your launchpad
I am on a launchpad mini, which is why I have such a weird configuration:

```
const LAUNCHPAD_ORDER: [u8; 64] = [ // LAUNCHPAD_ORDER[base_pitch_multiplier-1] = note, e.g. LAUNCH_PAD_ORDER[1-1] = 112 so note 112 will play 1 * the base pitch
    112, 113, 114, 115, 116, 117, 118, 119,
     96,  97,  98,  99, 100, 101, 102, 103,
     80,  81,  82,  83,  84,  85,  86,  87,
     64,  65,  66,  67,  68,  69,  70,  71,
     48,  49,  50,  51,  52,  53,  54,  55,
     32,  33,  34,  35,  36,  37,  38,  39,
     16,  17,  18,  19,  20,  21,  22,  23,
      0,   1,   2,   3,   4,   5,   6,   7
];

const LAUNCHPAD_RIGHT_SIDE: [u8; 8] = [8, 24, 40, 56, 72, 88, 104, 120]; // NOTEON
const RIGHT_SIDE_MULTIPLIERS: [f32; 8] = [9./8., 8./7., 7./6., 6./5., 5./4., 4./3., 3./2., 2.];
const LAUNCHPAD_TOP_SIDE: [u8; 8] = [104, 105, 106, 107, 108, 109, 110, 111]; // CC
const TOP_SIDE_MULTIPLIERS: [f32; 8] = [1./2., 2./3., 3./4., 4./5., 5./6., 6./7., 7./8., 8./9.];

const MPE_PITCH_BEND_RANGE: f32 = 48.0; // In semitones
```

To edit these constants for your launchpad, get something to monitor midi (e.g. [this vst](https://plugins4free.com/plugin/1432/)) and play all the notes in the middle square of the launchpad starting with the one you want to be the bottom note and moving up. I started with the bottom left note, which was 112, and then went through the whole row, and went up the rows one at a time. This should fill up `LAUNCHPAD_ORDER` with 64 numbers. `LAUNCHPAD_RIGHT_SIDE` was made by pressing all the right buttons starting from the top and moving down, and `LAUNCHPAD_TOP_SIDE` was made by pressing all the top buttons starting from the left and moving right, although my launchpad treats right notes as NOTEON signals and top notes as CC. If your top buttons are not CC or your right notes are not NOTEON, you will need to change the code.

Feel free to change the multipliers, they just multiply all frequencies by the number you put there when the matching button is pressed.

## build
```
rustup default nightly
rustup run nightly cargo build
```