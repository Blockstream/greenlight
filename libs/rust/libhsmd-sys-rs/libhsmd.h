#ifndef LIGHTNING_CONTRIB_LIBHSMD_SYS_RS_LIBHSMD_H
#define LIGHTNING_CONTRIB_LIBHSMD_SYS_RS_LIBHSMD_H

#include <hsmd/libhsmd.h>
u8 *c_handle(long long cap, long long dbid, const u8 *peer_id, size_t peer_id_len, u8 *msg, size_t msglen);
u8 *c_init(u8 *hsm_secret, char *network_name);

#endif /* LIGHTNING_CONTRIB_LIBHSMD_SYS_RS_LIBHSMD_H */
