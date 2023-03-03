no-midi
=======
A visualizer which sends MIDI commands.  These MIDI commands can be fed into a lighting controller to then control fancy event lighting equipment.

Events are sent as NOTE_ON/NOTE_OFF messages.  The following notes are mapped:

| MIDI Note | Meaning |
| --- | --- |
| 50 &lt;= n &lt; 60 | ON/OFF for the 10 frequency channels.  The 4 highest channels are ON, rest OFF.  I map this to 10 non-overlapping light scenes. |
| 66 | ON when a beat hits, OFF 100ms later unless a further beat immediately follows.  I like to trigger strobe lights on this. |
| 70 | Volume as note velocity. |
