// Render through the public AU API, without opening an editor or an audio device.
// Load the exact component argument into this process; never install/replace an AU.
#import <Foundation/Foundation.h>
#include <AudioToolbox/AudioToolbox.h>
#include <CoreMIDI/CoreMIDI.h>
#include <algorithm>
#include <cmath>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

namespace
{
constexpr double rate = 48000.0;
constexpr UInt32 block = 256;
void require(bool value, const std::string &message)
{
  if (!value) throw std::runtime_error(message);
}
void check(OSStatus status, const char *operation)
{
  require(status == noErr, std::string(operation) + ": OSStatus " + std::to_string(status));
}

enum class Input { bytes, ump1, ump2 };
const char *label(Input input)
{
  switch (input) {
    case Input::bytes: return "MIDI bytes";
    case Input::ump1: return "MIDI1 UMP";
    case Input::ump2: return "MIDI2 UMP";
  }
  return "unknown";
}

class Unit
{
  struct Owner {
    AudioUnit value = nullptr;
    ~Owner() {
      if (value) { AudioUnitUninitialize(value); AudioComponentInstanceDispose(value); }
    }
  } owner;
public:
  AudioUnit unit = nullptr;
  double time = 0;
  explicit Unit(AudioComponent component, double bendRange)
  {
    const auto created = AudioComponentInstanceNew(component, &unit);
    owner.value = unit;
    check(created, "instantiate AU");
    AudioStreamBasicDescription format{};
    format.mSampleRate = rate;
    format.mFormatID = kAudioFormatLinearPCM;
    format.mFormatFlags = kAudioFormatFlagsNativeFloatPacked | kAudioFormatFlagIsNonInterleaved;
    format.mBytesPerPacket = format.mBytesPerFrame = sizeof(float);
    format.mFramesPerPacket = 1;
    format.mChannelsPerFrame = 2;
    format.mBitsPerChannel = 32;
    check(AudioUnitSetProperty(unit, kAudioUnitProperty_StreamFormat, kAudioUnitScope_Output,
                               0, &format, sizeof(format)), "set output format");
    UInt32 maximum = block;
    check(AudioUnitSetProperty(unit, kAudioUnitProperty_MaximumFramesPerSlice,
                               kAudioUnitScope_Global, 0, &maximum, sizeof(maximum)), "set block size");
    check(AudioUnitInitialize(unit), "initialise AU");
    // Edit the plug-in's own state through the host API. This is a disposable
    // in-memory instance; there are no user presets or host projects involved.
    CFPropertyListRef state = nullptr;
    UInt32 size = sizeof(state);
    check(AudioUnitGetProperty(unit, kAudioUnitProperty_ClassInfo, kAudioUnitScope_Global,
                              0, &state, &size), "get AU state");
    NSDictionary *dictionary = CFBridgingRelease(state);
    NSData *data = dictionary[@"data"];
    require([data isKindOfClass:[NSData class]] && data.length > 8, "missing nih-plug state");
    NSData *jsonData = [data subdataWithRange:NSMakeRange(8, data.length - 8)];
    NSError *error = nil;
    NSMutableDictionary *json = [NSJSONSerialization JSONObjectWithData:jsonData
                                options:NSJSONReadingMutableContainers error:&error];
    require(json && !error && json[@"params"][@"midi_bend_range"][@"f32"],
            "state has no MIDI Bend Range");
    json[@"params"][@"midi_bend_range"][@"f32"] = @(bendRange);
    // A short release makes note-off/CC123 checks about MIDI delivery, rather
    // than the default patch's several-second analogue envelope tail.
    json[@"params"][@"loudness_decay"][@"f32"] = @0.0;
    NSData *encoded = [NSJSONSerialization dataWithJSONObject:json options:0 error:&error];
    require(encoded && !error, "cannot encode test state");
    uint64_t length = CFSwapInt64HostToLittle(encoded.length);
    NSMutableData *framed = [NSMutableData dataWithBytes:&length length:sizeof(length)];
    [framed appendData:encoded];
    NSMutableDictionary *replacement = [dictionary mutableCopy];
    replacement[@"data"] = framed;
    CFPropertyListRef replacementCF = (__bridge CFPropertyListRef)replacement;
    check(AudioUnitSetProperty(unit, kAudioUnitProperty_ClassInfo, kAudioUnitScope_Global, 0,
                               &replacementCF, sizeof(replacementCF)),
          "restore test bend range");
  }
  Unit(const Unit &) = delete;
  Unit &operator=(const Unit &) = delete;

  void midi(Input input, uint8_t status, uint8_t d1, uint8_t d2)
  {
    constexpr UInt32 offset = 31;
    if (input == Input::bytes) {
      check(MusicDeviceMIDIEvent(unit, status, d1, d2, offset), "MIDI bytes");
      return;
    }
    MIDIEventList list{};
    auto *packet = MIDIEventListInit(&list, input == Input::ump1 ? kMIDIProtocol_1_0 : kMIDIProtocol_2_0);
    UInt32 words[2] = {(0x2u << 28) | (UInt32(status) << 16) | (UInt32(d1) << 8) | d2, 0};
    UInt32 count = 1;
    if (input == Input::ump2) {
      words[0] = (0x4u << 28) | (UInt32(status) << 16) | (UInt32(d1) << 8);
      switch (status & 0xF0) {
        case 0x80: case 0x90: words[1] = UInt32(d2) << 25; break;
        case 0xB0: words[1] = UInt32(d2) << 25; break;
        case 0xE0:
          words[0] &= 0xFFFF0000u;
          words[1] = (UInt32(d1) | (UInt32(d2) << 7)) << 18;
          break;
        default: throw std::runtime_error("unsupported test MIDI2 message");
      }
      count = 2;
    }
    require(MIDIEventListAdd(&list, sizeof(list), packet, 0, count, words), "UMP packet capacity");
    if (@available(macOS 12.0, *)) {
      check(MusicDeviceMIDIEventList(unit, offset, &list), "MIDI event list");
    } else {
      throw std::runtime_error("MIDI event-list API requires macOS 12");
    }
  }

  std::vector<float> render(double seconds = 0.65)
  {
    std::vector<float> result;
    float left[block]{}, right[block]{};
    struct { UInt32 count; AudioBuffer buffers[2]; } buffers{
      2, {{1, sizeof(left), left}, {1, sizeof(right), right}}};
    for (size_t i = 0; i < size_t(seconds * rate); i += block) {
      AudioTimeStamp timestamp{};
      timestamp.mFlags = kAudioTimeStampSampleTimeValid;
      timestamp.mSampleTime = time;
      AudioUnitRenderActionFlags flags = 0;
      check(AudioUnitRender(unit, &flags, &timestamp, 0, block,
                            reinterpret_cast<AudioBufferList *>(&buffers)), "render AU");
      for (float sample : left) require(std::isfinite(sample), "non-finite AU output");
      result.insert(result.end(), left, left + block);
      time += block;
    }
    return result;
  }
};

double rms(const std::vector<float> &samples)
{
  double energy = 0;
  for (size_t i = samples.size() / 2; i < samples.size(); ++i) energy += samples[i] * samples[i];
  return std::sqrt(energy / (samples.size() - samples.size() / 2));
}

double frequency(const std::vector<float> &samples)
{
  require(rms(samples) > 1e-4, "note is missing/too quiet to measure");
  std::vector<double> crossings;
  for (size_t i = samples.size() / 2 + 1; i < samples.size(); ++i)
    if (samples[i-1] < 0 && samples[i] >= 0)
      crossings.push_back(double(i-1) - samples[i-1] / double(samples[i] - samples[i-1]));
  require(crossings.size() >= 20, "not enough periods to measure pitch");
  return rate * (crossings.size() - 1) / (crossings.back() - crossings.front());
}

void exercise(AudioComponent component, Input input, double range)
{
  Unit host(component, range);
  constexpr uint8_t channel = 3;
  auto bend = [&](uint16_t value) { host.midi(input, 0xE0 | channel, value & 127, value >> 7); };
  bend(8192);
  host.midi(input, 0x90 | channel, 69, 72);
  const auto initial = host.render();
  const double base = frequency(initial);
  const double onLevel = rms(initial);
  require(std::abs(1200 * std::log2(base / 440)) < 15, "A4 note delivery/tuning failed");
  for (uint16_t value : {16383, 8192, 0, 8192}) {
    bend(value);
    const double hz = frequency(host.render());
    const double cents = 1200 * std::log2(hz / base);
    const double expected = range * 100 * ((double(value) / 16383) * 2 - 1);
    std::cout << label(input) << ", range=" << range << ", bend=" << value
              << ", Hz=" << hz << ", cents=" << cents << ", expected=" << expected << '\n';
    require(std::abs(cents - expected) < 15, "pitch-bend direction/range/centering failed");
  }
  bend(16383);
  host.render();
  host.midi(input, 0xB0 | channel, 121, 0);
  require(std::abs(1200 * std::log2(frequency(host.render()) / base)) < 15,
          "CC121 did not centre pitch bend");
  host.midi(input, 0xB0 | channel, 123, 0);
  const double releaseLevel = rms(host.render(1.0));
  std::cout << label(input) << ", release RMS=" << releaseLevel << ", held RMS=" << onLevel << '\n';
  require(releaseLevel < onLevel * 0.01, "CC123 did not release the note by 40 dB");
  host.midi(input, 0x90 | channel, 69, 72);
  require(frequency(host.render()) > 400, "note delivery failed after CC123");
  host.midi(input, 0xB0 | channel, 120, 0);
  require(rms(host.render(0.5)) < 1e-4, "CC120 did not silence the note");
  host.midi(input, 0x90 | channel, 69, 72);
  require(frequency(host.render()) > 400, "note delivery failed after CC120");
  host.midi(input, 0x90 | channel, 69, 0);
  if (input == Input::ump2) {
    // MIDI2 zero-velocity note-on is a note, unlike MIDI1's note-off convention.
    require(frequency(host.render()) > 400, "MIDI2 zero velocity incorrectly released the note");
  } else {
    require(rms(host.render(1.0)) < onLevel * 0.01,
            "MIDI1 velocity-zero note-on did not release by 40 dB");
  }
  host.midi(input, 0x90 | channel, 69, 72);
  require(frequency(host.render()) > 400, "note delivery failed after velocity-zero release");
  host.midi(input, 0x80 | channel, 69, 0);
  require(rms(host.render(1.0)) < onLevel * 0.01, "note-off did not release by 40 dB");
  std::cout << "PASS " << label(input) << " range=" << range << " (bend/reset/notes/panic)\n";
}
} // namespace

int main(int argc, const char *argv[])
{
  @autoreleasepool {
    try {
      require(argc == 2, "usage: ca72-au-render-tests /absolute/path/CA-72.component");
      NSString *path = [NSString stringWithUTF8String:argv[1]];
      CFBundleRef bundle = CFBundleCreate(nullptr, (__bridge CFURLRef)[NSURL fileURLWithPath:path]);
      require(bundle && CFBundleLoadExecutable(bundle), "cannot load component bundle");
      const auto factory = reinterpret_cast<AudioComponentFactoryFunction>(
          CFBundleGetFunctionPointerForName(bundle, CFSTR("wrapAsAUV2_inst0Factory")));
      require(factory, "AU factory entry point missing");
      // A process-local identity ensures the system's installed AU cannot substitute for the argument.
      AudioComponentDescription description{'aumu', 'Tp43', 'IdlF', 0, 0};
      const auto component = AudioComponentRegister(&description, CFSTR("CA-72 MIDI regression"), 1, factory);
      require(component, "cannot register process-local AU");
      int failures = 0, cases = 0;
      for (Input input : {Input::bytes, Input::ump1, Input::ump2}) {
        if (input != Input::bytes) {
          if (@available(macOS 12.0, *)) { }
          else { std::cout << "SKIP " << label(input) << ": macOS 12 API unavailable\n"; continue; }
        }
        for (double range : {2.0, 7.0, 0.0}) {
          ++cases;
          try { exercise(component, input, range); }
          catch (const std::exception &e) {
            ++failures;
            std::cerr << "FAIL " << label(input) << " range=" << range << ": " << e.what() << '\n';
          }
        }
      }
      // The registry retains the factory pointer until process exit: keep its bundle loaded.
      std::cout << (failures ? "FAIL" : "PASS") << " AU MIDI render matrix (" << cases << " cases)\n";
      return failures ? 1 : 0;
    } catch (const std::exception &e) {
      std::cerr << "FAIL " << e.what() << '\n';
      return 1;
    }
  }
}
