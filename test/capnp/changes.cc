#include "gtest/gtest.h"
#include "fmt/core.h"

TEST(Dummy, Test)
{
    fmt::print("Hello World\n");
    EXPECT_TRUE(1);
}

