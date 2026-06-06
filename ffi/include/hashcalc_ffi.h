#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Returns the number of supported hash algorithms. */
int32_t hashcalc_algorithm_count(void);

/** Returns a static C string for the algorithm at @p index. Do NOT free it. */
const char* hashcalc_algorithm_name(int32_t index);

/** Hash a file without progress tracking. Free result with hashcalc_free_string. */
char* hashcalc_hash_file(const char* path, int32_t algo_index);

/**
 * Hash a file with progress tracking and cooperative cancellation.
 *
 * @param bytes_done  Pointer to a caller-owned, zero-initialised uint64_t.
 *                    Use std::atomic<uint64_t> on the C++ side.
 *                    Updated with relaxed atomics; safe to read from another thread.
 *                    May be NULL.
 *
 * @param file_size   Written once before reading starts.  May be NULL.
 *
 * @param cancelled   Pointer to a caller-owned uint8_t (std::atomic<uint8_t>).
 *                    Write 1 to request cancellation; the hasher stops at the
 *                    next 64 KiB chunk boundary and returns NULL.  May be NULL.
 *
 * @return  Heap-allocated hash string on success, NULL on error or cancellation.
 *          Free with hashcalc_free_string.
 */
char* hashcalc_hash_file_progress(const char*   path,
                                  int32_t       algo_index,
                                  uint64_t*     bytes_done,
                                  uint64_t*     file_size,
                                  const uint8_t* cancelled);

/** Frees a string returned by hashcalc_hash_file or hashcalc_hash_file_progress. */
void hashcalc_free_string(char* s);

#ifdef __cplusplus
}
#endif
