#pragma once

#include <elem/Runtime.h>
#include <memory>
#include <rust/cxx.h>
#include <string>
#include <vector>


// Re-binding to get a void* over the CXX FFI
using c_void = void;

class RuntimeBindings {
public:
    RuntimeBindings(double sampleRate, size_t blockSize);
    ~RuntimeBindings();

    int add_shared_resource(rust::String const& name, size_t numChannels, size_t numFrames, rust::Slice<float const> data);
    int apply_instructions(rust::String const& batch);
    rust::String process_queued_events();

    void process(float const* inputData, float* outputData, size_t numChannels, size_t numFrames, c_void* userData);

private:
    elem::Runtime<float> m_runtime;
};

std::unique_ptr<RuntimeBindings> new_runtime_instance(double sampleRate, size_t blockSize);
