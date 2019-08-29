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

INSTANTIATE_TEST_CASE_P(JsonCodec, ChangeStatusTest,
                        ::testing::Values(std::pair{ gerrit::ChangeStatus::NEW, "NEW" },
                                          std::pair{ gerrit::ChangeStatus::MERGED, "MERGED" },
                                          std::pair{ gerrit::ChangeStatus::ABANDONED, "ABANDONED" },
                                          std::pair{ gerrit::ChangeStatus::DRAFT, "DRAFT" }));

TEST_P(ChangeStatusTest, EncodeDecode)
{
    const gerrit::ChangeStatus kStatusEnum = GetParam().first;
    const std::string_view kStatusName = GetParam().second;
    const std::string status_quote = fmt::format("\"{}\"", kStatusName);

    capnp::JsonCodec codec;
    codec.handleByAnnotation<gerrit::ChangeStatus>();

    { /* Decode from JSON to Enum */
        auto status_enum = codec.decode({ status_quote.data(), status_quote.length() },
                                        capnp::EnumSchema::from<gerrit::ChangeStatus>());
        EXPECT_EQ(kStatusEnum, status_enum.as<gerrit::ChangeStatus>());
    }

    { /* Encode Enum to JSON */
        auto status_json = codec.encode(kStatusEnum);
        EXPECT_STREQ(status_quote.c_str(), status_json.cStr());
    }
}
