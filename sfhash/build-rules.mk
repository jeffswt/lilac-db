# build smhasher tests

CC          = gcc
CXX         = g++
CPPFLAGS    = -O2
CFLAGS      = -std=c99
CXXFLAGS    = -std=c++11
LDFLAGS     = -O2

BUILD_DIR   = ./target
SRC_DIRS    = ./src
TARGET_EXEC = smhasher

F_SRCS      = $(shell find $(SRC_DIRS) -name '*.cpp' -or -name '*.c')
F_OBJS      = $(F_SRCS:%=$(BUILD_DIR)/%.o)


# rule for c code
$(BUILD_DIR)/%.c.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c $< -o $@

# rule for c++ code
$(BUILD_DIR)/%.cpp.o: %.cpp
	@mkdir -p $(dir $@)
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

# final linking
$(BUILD_DIR)/$(TARGET_EXEC): $(F_OBJS)
	$(CXX) $(F_OBJS) -o $@ $(LDFLAGS)

all: $(BUILD_DIR)/$(TARGET_EXEC)
