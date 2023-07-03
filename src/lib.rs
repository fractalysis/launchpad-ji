use nih_plug::prelude::*;
use std::sync::Arc;


//   0   1   2   3   4   5   6   7     8
//  16  17  18  19  20  21  22  23    24
//  32  33  34  35  36  37  38  39    40
//  48  49  50  51  52  53  54  55    56
//  64  65  66  67  68  69  70  71    72
//  80  81  82  83  84  85  86  87    88
//  96  97  98  99 100 101 102 103   104
// 112 113 114 115 116 117 118 119   120

const LAUNCHPAD_ORDER: [u8; 64] = [ // LAUNCHPAD_ORDER[base_pitch_multiplier-1] = note, e.g. LAUNCH_PAD_ORDER[0] = 112 so note 112 will play 1 * the base pitch
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
const RIGHT_SIDE_MULTIPLIERS: [f32; 8] = [2., 3./2., 4./3., 5./4., 6./5., 7./6., 8./7., 9./8.];
const LAUNCHPAD_TOP_SIDE: [u8; 8] = [104, 105, 106, 107, 108, 109, 110, 111]; // CC
const TOP_SIDE_MULTIPLIERS: [f32; 8] = [1./2., 2./3., 3./4., 4./5., 5./6., 6./7., 7./8., 8./9.];


struct LaunchpadJI {
    params: Arc<LaunchpadJIParams>,

    // CUSTOM
    channel_voices: [Option<u8>; 16],
    right_side_notes: [bool; 8],
    top_side_notes: [bool; 8],
    current_multiplier: f32,
}

#[derive(Params)]
struct LaunchpadJIParams {
    #[id = "base_pitch"]
    base_pitch: FloatParam
}

impl Default for LaunchpadJIParams {
    fn default() -> Self {
        LaunchpadJIParams {
            base_pitch: FloatParam::new("base_pitch", 41.2, FloatRange::Skewed { min: 20.0, max: 200.0, factor: 2.0 }) // Start on E1
        }
    }
}

impl Default for LaunchpadJI {
    fn default() -> Self {
        Self {
            params: Arc::new(LaunchpadJIParams::default()),

            channel_voices: [Some(0); 16],
            right_side_notes: [false; 8],
            top_side_notes: [false; 8],
            current_multiplier: 1.0
        }
    }
}

impl Plugin for LaunchpadJI {
    const NAME: &'static str = "Launchpad JI";
    const VENDOR: &'static str = "Fractalysis";
    const URL: &'static str = "";
    const EMAIL: &'static str = "fractalysisofficial@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // This plugin doesn't have any audio IO
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Some(event) = context.next_event() {
            match event {

                NoteEvent::NoteOn {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    let center_note_option = LAUNCHPAD_ORDER.into_iter().position(|v| v == note);
                    match center_note_option {
                        Some(center_note) => {
                            // Assign note to next available channel
                        }
                        None => {}
                    }

                    let right_note_option = LAUNCHPAD_RIGHT_SIDE.into_iter().position(|v| v == note);
                    match right_note_option {
                        Some(right_note) => {
                            // Update the array of notes that are held down
                            self.right_side_notes[right_note] = true;

                            // Update multiplier
                            // ...

                            // Update all channel pitch bends
                            // ...
                        }
                        None => {}
                    }
                }

                NoteEvent::NoteOff {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {

                    let center_note_option = self.channel_voices.into_iter().position(|v| v == Some(note));
                    match center_note_option {
                        Some(center_note) => {
                            // Free the channel
                            self.channel_voices[center_note] = None;

                            // Update multiplier
                            // ...

                            // Update all channel pitch bends
                            // ...
                        }
                        None => {}
                    }

                    let right_note_option = LAUNCHPAD_RIGHT_SIDE.into_iter().position(|v| v == note);
                    match right_note_option {
                        Some(right_note) => {
                            // Update the array of notes that are held down
                            self.right_side_notes[right_note] = false;
                        }
                        None => {}
                    }
                }

                NoteEvent::MidiCC {
                    timing,
                    channel,
                    cc,
                    value,
                } => {
                    // 104 105 106 107 108 109 110 111 (top row)

                    let top_note_option = LAUNCHPAD_TOP_SIDE.into_iter().position(|v| v == cc);
                    match top_note_option {
                        Some(top_note) => {
                            // Update the array of notes that are held down
                            if value > 0. {
                                self.top_side_notes[top_note] = true;
                            }
                            else{
                                self.top_side_notes[top_note] = false;
                            }
                        }
                        None => {}
                    }

                }

                _ => ()

            }
        }

        ProcessStatus::Normal
    }
}


impl Vst3Plugin for LaunchpadJI {
    const VST3_CLASS_ID: [u8; 16] = *b"LaunchpadMPEJIfx";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_vst3!(LaunchpadJI);