#include "clankers.h"

#include <stddef.h>

/* Compile-only smoke test: every public symbol is declared and enums are usable. */
static void smoke(void) {
    const char *ver = clankers_version();
    (void)ver;
    (void)clankers_abi_version();
    (void)clankers_last_error_code();
    (void)clankers_last_error_message();

    ClankersShape shape = {0};
    shape.rank = 2;
    shape.dims[0] = 1;
    shape.dims[1] = 4;

    ClankersTensorView view = {0};
    view.dtype = CLANKERS_D_TYPE_F32;
    view.layout = CLANKERS_LAYOUT_CONTIGUOUS;
    view.device = CLANKERS_DEVICE_CPU;
    view.shape = shape;

    ClankersInferenceStats stats = {0};
    (void)stats;
    (void)view;
}

int main(void) {
    smoke();
    return 0;
}
