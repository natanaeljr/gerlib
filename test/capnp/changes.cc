/**
 * \file
 * \brief Test for Changes capnp.
 **/

#include "gtest/gtest.h"
#include <string>
#include <tuple>

#include "fmt/format.h"
#include "ger/changes.capnp.h"
#include "capnp/compat/json.h"

class ChangeStatusTest
    : public ::testing::TestWithParam<std::pair<gerrit::ChangeStatus, std::string_view>> {
};

INSTANTIATE_TEST_CASE_P(
    JsonCodec, ChangeStatusTest,
    ::testing::Values(std::pair{ gerrit::ChangeStatus::NEW, "NEW" },
                      std::pair{ gerrit::ChangeStatus::MERGED, "MERGED" },
                      std::pair{ gerrit::ChangeStatus::ABANDONED, "ABANDONED" },
                      std::pair{ gerrit::ChangeStatus::DRAFT, "DRAFT" }));

TEST_P(ChangeStatusTest, EncodeDecode)
{
    const gerrit::ChangeStatus kStatus = GetParam().first;
    const std::string_view kStatusName = GetParam().second;

    capnp::MallocMessageBuilder arena;
    capnp::JsonCodec codec;
    codec.handleByAnnotation<gerrit::ChangeStatus>();

    /* Decode from JSON to Enum */
    const std::string in_json = fmt::format(R"({{"status":"{}"}})", kStatusName);
    auto orphan = codec.decode<gerrit::ChangeTest>({ in_json.data(), in_json.length() },
                                                   arena.getOrphanage());
    EXPECT_EQ(kStatus, orphan.getReader().getStatus());

    /* Encode Enum to JSON */
    auto builder = arena.initRoot<gerrit::ChangeTest>();
    builder.setStatus(kStatus);
    auto out_json = codec.encode(builder);
    EXPECT_STREQ(fmt::format(R"({{"status":"{}"}})", kStatusName).c_str(), out_json.cStr());
}
