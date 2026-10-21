#ifndef PACKVIUM_H
#define PACKVIUM_H
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif

/* Returned strings are UTF-8 and must be released with packvium_free_string. */
char *packvium_version(void);
char *packvium_solve_json(const char *request_json);
char *packvium_rebalance_json(const char *request_json, const char *result_json, size_t max_moves);
char *packvium_commerce_quote(const char *call_json);
char *packvium_commerce_evaluate_policy(const char *call_json);
char *packvium_commerce_catalog_version_info(const char *call_json);
void packvium_free_string(char *value);

#ifdef __cplusplus
}
#endif
#endif
