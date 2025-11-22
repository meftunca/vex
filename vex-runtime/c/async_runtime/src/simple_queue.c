// Simple mutex-based MPMC queue (temporary workaround for lockfree ABA bug)
#include <stdlib.h>
#include <stdio.h>
#include <pthread.h>
#include <stdbool.h>
#include "internal.h"

typedef struct QNode
{
  void *data;
  struct QNode *next;
} QNode;

typedef struct
{
  pthread_mutex_t lock;
  QNode *head;
  QNode *tail;
} SimpleQueue;

LockFreeQueue *lfq_create(size_t capacity)
{
  (void)capacity;
  SimpleQueue *q = (SimpleQueue *)xmalloc(sizeof(SimpleQueue));
  pthread_mutex_init(&q->lock, NULL);
  q->head = q->tail = NULL;
  return (LockFreeQueue *)q;
}

void lfq_destroy(LockFreeQueue *lfq)
{
  SimpleQueue *q = (SimpleQueue *)lfq;
  pthread_mutex_lock(&q->lock);
  QNode *n = q->head;
  while (n)
  {
    QNode *next = n->next;
    xfree(n);
    n = next;
  }
  pthread_mutex_unlock(&q->lock);
  pthread_mutex_destroy(&q->lock);
  xfree(q);
}

bool lfq_enqueue(LockFreeQueue *lfq, void *data)
{
  SimpleQueue *q = (SimpleQueue *)lfq;
  QNode *node = (QNode *)xmalloc(sizeof(QNode));
  node->data = data;
  node->next = NULL;

  pthread_mutex_lock(&q->lock);
  if (q->tail)
  {
    q->tail->next = node;
    q->tail = node;
  }
  else
  {
    q->head = q->tail = node;
  }
  pthread_mutex_unlock(&q->lock);
  return true;
}

bool lfq_dequeue(LockFreeQueue *lfq, void **out)
{
  SimpleQueue *q = (SimpleQueue *)lfq;
  pthread_mutex_lock(&q->lock);
  QNode *node = q->head;
  if (!node)
  {
    pthread_mutex_unlock(&q->lock);
    return false;
  }
  q->head = node->next;
  if (!q->head)
    q->tail = NULL;
  *out = node->data;
  pthread_mutex_unlock(&q->lock);
  xfree(node);
  return true;
}
