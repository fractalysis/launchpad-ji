extern crate baseplug;

use serde::{Deserialize, Serialize};
use baseplug::event::*;
use baseplug::*;
use ringbuf::Rb;
use ringbuf::StaticRb;



// 104 105 106 107 108 109 110 111 (<- CC)
//   0   1   2   3   4   5   6   7     8
//  16  17  18  19  20  21  22  23    24
//  32  33  34  35  36  37  38  39    40
//  48  49  50  51  52  53  54  55    56
//  64  65  66  67  68  69  70  71    72
//  80  81  82  83  84  85  86  87    88
//  96  97  98  99 100 101 102 103   104
// 112 113 114 115 116 117 118 119   120

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



// PLUGIN DEFINITION


baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct LaunchpadJIParams {
        #[model(min = 8.0, max = 128.0)]
        #[parameter(name = "Base Frequency",
            gradient = "Power(2.0)")]
        base_frequency: f32,
    }
}

impl Default for LaunchpadJIParams {
    fn default() -> Self {
        Self {
            base_frequency: 20.6, // E
        }
    }
}

struct LaunchpadJI {
    current_multiplier: f32,
    channel_voices: [Option<u8>; 16],
    right_side_notes: [bool; 8],
    top_side_notes: [bool; 8],

    midi_queue: StaticRb::<Event<LaunchpadJI>, 32>,
}

impl Plugin for LaunchpadJI {
    const NAME: &'static str = "Launchpad JI";
    const PRODUCT: &'static str = "Launchpad JI";
    const VENDOR: &'static str = "Fractalysoft";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = LaunchpadJIParams;

    #[inline]
    fn new(_sample_rate: f32, _model: &LaunchpadJIParams) -> Self {
        
        let mut to_ret = LaunchpadJI {
            current_multiplier: 1.,
            channel_voices: [None; 16],
            right_side_notes: [false; 8],
            top_side_notes: [false; 8],

            midi_queue: StaticRb::default(),
        };

        // MPE initialization

        to_ret.midi_queue.push(Event::<LaunchpadJI> {
            frame: 0,
            data: Data::Midi([0xB0, 0x79, 0x00]),
        });

        to_ret.midi_queue.push(Event::<LaunchpadJI> {
            frame: 0,
            data: Data::Midi([0xB0, 0x64, 0x06]),
        });

        to_ret.midi_queue.push(Event::<LaunchpadJI> {
            frame: 0,
            data: Data::Midi([0xB0, 0x65, 0x00]),
        });

        to_ret.midi_queue.push(Event::<LaunchpadJI> {
            frame: 0,
            data: Data::Midi([0xB0, 0x06, 0x0F]), // 15 (0F) channels
        });

        to_ret
    }

    // Do nothing to the audio
    #[inline]
    fn process(&mut self, model: &LaunchpadJIParamsProcess, ctx: &mut ProcessContext<Self>) {

        // Send all midi events
        let enqueue_midi = &mut ctx.enqueue_event;
        while let Some(event) = self.midi_queue.pop() {
            enqueue_midi(event);
        }

        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            output[0][i] = input[0][i];
            output[1][i] = input[1][i];
        }
    }
}



// MIDI PROCESSING


impl LaunchpadJI {
    fn update_pitch_bend(&mut self, _model: &LaunchpadJIParamsProcess){

        // Update multiplier
        self.current_multiplier = 1.;

        for (index, down) in self.right_side_notes.iter().enumerate() {
            if *down {
                self.current_multiplier *= RIGHT_SIDE_MULTIPLIERS[index];
            }
        }

        for (index, down) in self.top_side_notes.iter().enumerate() {
            if *down {
                self.current_multiplier *= TOP_SIDE_MULTIPLIERS[index];
            }
        }

        // Update all channel pitch bends
        for (channel_index, note_option) in self.channel_voices.iter().enumerate() {
            match note_option {
                Some(note) => {
                    let pitch_bend_bytes = self.get_mapped_pitch_bend(*note, _model);

                    let pitch_bend = Event::<LaunchpadJI> {
                        frame: 0,
                        data: Data::Midi([(0xE0 + channel_index).try_into().unwrap(), pitch_bend_bytes[0], pitch_bend_bytes[1]]),
                    };
                    self.midi_queue.push(pitch_bend);
                }

                None => {}
            }
        }
    }

    fn get_mapped_note(&self, note: u8, _model: &LaunchpadJIParamsProcess) -> u8 {
        let freq = _model.base_frequency.values.last().unwrap() * (LAUNCHPAD_ORDER.into_iter().position(|v| v == note).unwrap() + 1) as f32;
        let pitch = 12.0 * (freq/440.0).log2() + 69.0;
        return pitch.round() as u8;
    }

    fn get_mapped_pitch_bend(&self, note: u8, _model: &LaunchpadJIParamsProcess) -> [u8; 2] {
        let freq = _model.base_frequency.values.last().unwrap() * (LAUNCHPAD_ORDER.into_iter().position(|v| v == note).unwrap() + 1) as f32;
        let pitch = 12.0 * (freq * self.current_multiplier/440.0).log2() + 69.0;
        let pitch_bend = pitch - self.get_mapped_note(note, _model) as f32;
        // pitch bend should be (-1, 1)

        // convert to 14 bit midi pitch bend
        let midi_value = ((pitch_bend * 8192.0 / MPE_PITCH_BEND_RANGE) + 8192.0) as u16;
    
        // Extract the least significant and most significant bytes
        let lsb = (midi_value & 0x7F) as u8;
        let msb = ((midi_value >> 7) & 0x7F) as u8;

        [lsb, msb]
    }
}

impl MidiReceiver for LaunchpadJI {
    fn midi_input(&mut self, _model: &LaunchpadJIParamsProcess, msg: [u8; 3]) {
        match msg[0] {
            // note on
            0x90 => {
                let note = msg[1];

                let center_note_option = LAUNCHPAD_ORDER.into_iter().position(|v| v == note);

                match center_note_option {
                    
                    Some(center_note) => {

                        // Assign note to next available channel
                        for (channel_index, channel_option) in self.channel_voices[1 .. 16].iter_mut().enumerate(){
                            if channel_option.is_none() {
                                *channel_option = Some(note);

                                // Send midi input to channel_index

                                let pitch_bend_bytes = self.get_mapped_pitch_bend(note, _model);
                                let pitch_bend = Event::<LaunchpadJI> {
                                    frame: 0,
                                    data: Data::Midi([(0xE1 + channel_index).try_into().unwrap(), pitch_bend_bytes[0], pitch_bend_bytes[1]]),
                                };
                                self.midi_queue.push(pitch_bend);

                                let note_on = Event::<LaunchpadJI> {
                                    frame: 0,
                                    data: Data::Midi([(0x91 + channel_index).try_into().unwrap(), self.get_mapped_note(note, _model), msg[2]]),
                                };
                                self.midi_queue.push(note_on);

                                return;
                            }
                        }
                    }
                    None => {}
                }
                
                let right_note_option = LAUNCHPAD_RIGHT_SIDE.into_iter().position(|v| v == note);
                match right_note_option {
                    Some(right_note) => {
                        // Update the array of notes that are held down
                        self.right_side_notes[right_note] = true;

                        self.update_pitch_bend(_model);
                        
                        return;
                    }
                    None => {}
                }
            },

            // note off
            0x80 => {
                let note = msg[1];

                let channel_option = self.channel_voices.into_iter().position(|v| v == Some(note));
                match channel_option {
                    Some(channel_index) => {
                        // Free the channel
                        self.channel_voices[channel_index] = None;

                        // Stop the note on channel_index
                        let note_off = Event::<LaunchpadJI> {
                            frame: 0,
                            data: Data::Midi([(0x80 + channel_index).try_into().unwrap(), self.get_mapped_note(note, _model), msg[2]])
                        };
                        self.midi_queue.push(note_off);

                        return;
                    }
                    None => {}
                }

                let right_note_option = LAUNCHPAD_RIGHT_SIDE.into_iter().position(|v| v == note);
                match right_note_option {
                    Some(right_note) => {
                        // Update the array of notes that are held down
                        self.right_side_notes[right_note] = false;

                        self.update_pitch_bend(_model);

                        return;
                    }
                    None => {}
                }
            },

            // control change
            0xB0 => {
                let cc = msg[1];
                let value = msg[2];
                // 104 105 106 107 108 109 110 111 (top row)

                let top_note_option = LAUNCHPAD_TOP_SIDE.into_iter().position(|v| v == cc);
                match top_note_option {
                    Some(top_note) => {
                        // Update the array of notes that are held down
                        if value > 0 {
                            self.top_side_notes[top_note] = true;
                        }
                        else{
                            self.top_side_notes[top_note] = false;
                        }

                        self.update_pitch_bend(_model);
                        
                        return;
                    }
                    None => {}
                }
            }

            _ => ()
        }
    }
}



baseplug::vst2!(LaunchpadJI, b"FRlj");