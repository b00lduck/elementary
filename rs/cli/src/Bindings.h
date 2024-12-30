#pragma once

#include <elem/Runtime.h>
#include <memory>
#include <rust/cxx.h>
#include <string>
#include <vector>


class RuntimeBindings {
public:
    RuntimeBindings(double sampleRate, size_t blockSize);
    ~RuntimeBindings();

    int apply_instructions(rust::String const& batch);
    void process(float const* inputData, float* outputData, size_t numChannels, size_t numFrames);

private:
    std::vector<float> scratchData;
    elem::Runtime<float> m_runtime;
};

std::unique_ptr<RuntimeBindings> new_runtime_instance(double sampleRate, size_t blockSize);
