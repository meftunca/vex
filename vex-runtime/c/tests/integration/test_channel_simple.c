#include "vex_channel.h"
#include <stdio.h>

int main()
{
  // Create channel with capacity 4
  vex_channel_t *ch = vex_channel_create(4);
  if (!ch)
  {
    printf("Failed to create channel\n");
    return 1;
  }

  // Send two i64 values
  int64_t val1 = 10;
  int64_t val2 = 20;

  vex_channel_status_t status;
  status = vex_channel_send(ch, &val1);
  if (status != VEX_CHANNEL_OK)
  {
    printf("Send val1 failed: %d\n", status);
    return 1;
  }

  status = vex_channel_send(ch, &val2);
  if (status != VEX_CHANNEL_OK)
  {
    printf("Send val2 failed: %d\n", status);
    return 1;
  }

  // Receive two values
  void *recv1_ptr = NULL;
  void *recv2_ptr = NULL;

  status = vex_channel_recv(ch, &recv1_ptr);
  if (status != VEX_CHANNEL_OK)
  {
    printf("Recv val1 failed: %d\n", status);
    return 1;
  }

  status = vex_channel_recv(ch, &recv2_ptr);
  if (status != VEX_CHANNEL_OK)
  {
    printf("Recv val2 failed: %d\n", status);
    return 1;
  }

  // Extract values
  int64_t *p1 = (int64_t *)recv1_ptr;
  int64_t *p2 = (int64_t *)recv2_ptr;

  printf("Received: %lld, %lld\n", *p1, *p2);

  if (*p1 != 10 || *p2 != 20)
  {
    printf("FAIL: Expected 10, 20\n");
    return 1;
  }

  printf("SUCCESS!\n");
  vex_channel_destroy(ch);
  return 0;
}
