/**
 * \file
 * \brief Test for Changes capnp.
 **/

#include "gtest/gtest.h"
#include <string_view>

#include "fmt/core.h"
#include "ger/changes.capnp.h"
#include "capnp/compat/json.h"

TEST(JsonCodec, ChangeStatus)
{
    constexpr std::string_view input = R"({"status":"MERGED"})";
    capnp::MallocMessageBuilder arena;
    capnp::JsonCodec codec;
    codec.handleByAnnotation<gerrit::ChangeStatus>();

    /* Decode ChangeStatus from json */
    auto orphan = codec.decode<gerrit::ChangeTest>({ input.begin(), input.end() },
                                                   arena.getOrphanage());
    EXPECT_EQ(gerrit::ChangeStatus::MERGED, orphan.getReader().getStatus());

    /* Encode ChangeStatus to json */
    auto builder = arena.initRoot<gerrit::ChangeTest>();
    builder.setStatus(gerrit::ChangeStatus::DRAFT);
    auto output = codec.encode(builder);
    EXPECT_STREQ(R"({"status":"DRAFT"})", output.cStr());
}
