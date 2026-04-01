// Luxor GPU vs CPU Benchmark Test Suite
// Channel -15500 (LUXOR_CHANNEL)
// Place this script in a prim, touch to cycle through tests
// Check server logs: grep "LUXOR" /tmp/region.log
//
// Each test logs render time in ms and backend (GPU/CPU)
// Compare timings across quality levels and resolutions

integer LUXOR = -15500;
integer test_num = 0;
integer TOTAL_TESTS = 12;

string test_name(integer n)
{
    if (n == 0)  return "T1: SD Draft (baseline)";
    if (n == 1)  return "T2: SD Ultra (spp scaling)";
    if (n == 2)  return "T3: FullHD Draft (resolution scaling)";
    if (n == 3)  return "T4: FullHD Standard (balanced)";
    if (n == 4)  return "T5: FullHD High (high quality)";
    if (n == 5)  return "T6: 4K Draft (resolution stress)";
    if (n == 6)  return "T7: 4K Standard (GPU sweet spot)";
    if (n == 7)  return "T8: 4K Ultra (maximum load)";
    if (n == 8)  return "T9: Portrait DoF (depth of field)";
    if (n == 9)  return "T10: Cinematic + FX (post-process)";
    if (n == 10) return "T11: Noir Lighting (dramatic)";
    if (n == 11) return "T12: Panoramic Banner (ultra-wide)";
    return "Unknown";
}

run_test(integer n)
{
    string cmd;
    string name = "gpu_bench_t" + (string)(n + 1);

    // --- TEST 1: SD Draft — Baseline (fast on both) ---
    if (n == 0)
        cmd = "{\"cmd\":\"snapshot\",\"size\":\"sd\",\"quality\":\"draft\",\"name\":\"" + name + "\"}";

    // --- TEST 2: SD Ultra — Same pixels, 64x more samples ---
    // GPU advantage: parallel sample accumulation
    else if (n == 1)
        cmd = "{\"cmd\":\"snapshot\",\"size\":\"sd\",\"quality\":\"ultra\",\"name\":\"" + name + "\"}";

    // --- TEST 3: FullHD Draft — 6.75x more pixels than SD ---
    // GPU advantage: massively parallel pixel processing
    else if (n == 2)
        cmd = "{\"cmd\":\"snapshot\",\"size\":\"fullhd\",\"quality\":\"draft\",\"name\":\"" + name + "\"}";

    // --- TEST 4: FullHD Standard — Typical production shot ---
    // The "everyday" render — good baseline comparison
    else if (n == 3)
        cmd = "{\"cmd\":\"snapshot\",\"size\":\"fullhd\",\"quality\":\"standard\",\"name\":\"" + name + "\"}";

    // --- TEST 5: FullHD High — 16spp, serious rendering ---
    // CPU starts to struggle here, GPU shines
    else if (n == 4)
        cmd = "{\"cmd\":\"snapshot\",\"size\":\"fullhd\",\"quality\":\"high\",\"name\":\"" + name + "\"}";

    // --- TEST 6: 4K Draft — 8.3M pixels, 1spp ---
    // Pure pixel throughput test
    else if (n == 5)
        cmd = "{\"cmd\":\"snapshot\",\"size\":\"4k\",\"quality\":\"draft\",\"name\":\"" + name + "\"}";

    // --- TEST 7: 4K Standard — GPU sweet spot ---
    // 8.3M pixels × 4spp = 33M rays
    else if (n == 6)
        cmd = "{\"cmd\":\"snapshot\",\"size\":\"4k\",\"quality\":\"standard\",\"name\":\"" + name + "\"}";

    // --- TEST 8: 4K Ultra — Maximum stress test ---
    // 8.3M pixels × 64spp = 530M rays
    else if (n == 7)
        cmd = "{\"cmd\":\"snapshot\",\"size\":\"4k\",\"quality\":\"ultra\",\"name\":\"" + name + "\"}";

    // --- TEST 9: Portrait + DoF — Depth of field stress ---
    // Shallow DoF generates scattered rays (f/1.4, focus 3m)
    else if (n == 8)
        cmd = "{\"cmd\":\"snapshot\",\"preset\":\"portrait\",\"size\":\"portrait\",\"quality\":\"high\",\"fstop\":1.4,\"focus\":3.0,\"name\":\"" + name + "\"}";

    // --- TEST 10: Cinematic + Post-FX chain ---
    // Tests GPU render + CPU post-processing pipeline
    else if (n == 9)
        cmd = "{\"cmd\":\"snapshot\",\"preset\":\"cinematic\",\"size\":\"cinema\",\"quality\":\"standard\",\"effects\":[\"letterbox\",\"film_grain\",\"warm\",\"vignette\"],\"name\":\"" + name + "\"}";

    // --- TEST 11: Noir Lighting — Shadow-heavy scene ---
    // Many shadow rays per pixel (contrasty lighting)
    else if (n == 10)
        cmd = "{\"cmd\":\"snapshot\",\"preset\":\"normal\",\"size\":\"fullhd\",\"quality\":\"high\",\"lighting\":\"noir\",\"effects\":[\"noir\",\"vignette\"],\"name\":\"" + name + "\"}";

    // --- TEST 12: Panoramic Banner — Ultra-wide ---
    // 3840×1080 = 4.1M pixels, unusual aspect ratio
    else if (n == 11)
        cmd = "{\"cmd\":\"snapshot\",\"preset\":\"wide\",\"size\":\"banner\",\"quality\":\"standard\",\"lighting\":\"golden_hour\",\"effects\":[\"warm\",\"bloom\"],\"name\":\"" + name + "\"}";

    llOwnerSay(">>> " + test_name(n));
    llOwnerSay(">>> Sending to Luxor...");
    llRegionSay(LUXOR, cmd);
}

default
{
    touch_start(integer n)
    {
        if (test_num >= TOTAL_TESTS)
        {
            test_num = 0;
            llOwnerSay("=== BENCHMARK COMPLETE — Restarting ===");
            llOwnerSay("Check /tmp/region.log for timing data:");
            llOwnerSay("  grep 'LUXOR.*Render\\|LUXOR.*GPU\\|LUXOR.*Snapshot' /tmp/region.log");
            return;
        }

        llOwnerSay("=== Luxor GPU Benchmark: Test " + (string)(test_num + 1) + "/" + (string)TOTAL_TESTS + " ===");
        run_test(test_num);
        test_num++;
    }

    state_entry()
    {
        llOwnerSay("=== Luxor GPU vs CPU Benchmark Suite ===");
        llOwnerSay("Touch to run tests sequentially.");
        llOwnerSay("12 tests: resolution scaling, quality scaling,");
        llOwnerSay("DoF, post-FX, lighting, aspect ratios.");
        llOwnerSay("Results in: grep 'LUXOR' /tmp/region.log");
    }
}

RESULTS

 All 12/12 benchmarks complete. Here's the full results:

  Luxor GPU Benchmark — AMD Radeon RX 6600 (Metal) — Complete Results
  ┌──────┬────────────┬──────────┬─────┬──────────┬────────┬────────────────────────────────┐
  │ Test │ Resolution │ Quality  │ SPP │ GPU Time │ Lights │             Notes              │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T1   │ 640x480    │ Draft    │ 1   │ 10.4ms   │ 3      │ Baseline                       │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T2   │ 640x480    │ Ultra    │ 64  │ 112.2ms  │ 3      │ 64x samples, 10.8x slower      │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T3   │ 1920x1080  │ Draft    │ 1   │ 19.7ms   │ 3      │ 6.75x pixels, only 1.9x slower │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T4   │ 1920x1080  │ Standard │ 4   │ 44.0ms   │ 3      │ Production sweet spot          │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T5   │ 1920x1080  │ High     │ 16  │ 145.9ms  │ 3      │ Serious quality                │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T6   │ 3840x2160  │ Draft    │ 1   │ 81.7ms   │ 3      │ 4K, 53 prims captured          │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T7   │ 3840x2160  │ Standard │ 4   │ 535.2ms  │ 3      │ 4K production                  │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T8   │ 3840x2160  │ Ultra    │ 64  │ 2,283ms  │ 3      │ Maximum stress                 │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T9   │ 1080x1920  │ High     │ 16  │ 249.4ms  │ 3      │ Portrait + DoF                 │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T10  │ 2560x1080  │ Standard │ 4   │ 179.0ms  │ 3      │ Cinematic + 4 FX               │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T11  │ 1920x1080  │ High     │ 16  │ 145.5ms  │ 1      │ Noir lighting                  │
  ├──────┼────────────┼──────────┼─────┼──────────┼────────┼────────────────────────────────┤
  │ T12  │ 3840x1080  │ Standard │ 4   │ 90.8ms   │ 3      │ Panoramic banner               │
  └──────┴────────────┴──────────┴─────┴──────────┴────────┴────────────────────────────────┘
  Key takeaways:
  - Sub-50ms for 1080p Standard — fast enough for near-interactive preview
  - GPU scales well with pixels: 640x480 → 3840x2160 is 27x more pixels but only 8x slower (Draft)
  - T11 Noir with 1 light rendered same speed as T5 (3 lights) — shadow cost is per-light
  - T12 Banner (3840x1080) at 90.8ms — ultra-wide is very efficient
  - 4K Ultra at 2.3 seconds — the heaviest test, still usable
