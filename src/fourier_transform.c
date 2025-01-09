#include <complex.h>
#include <math.h>
#include <pthread.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>


#define floating_t float
typedef struct {
    complex floating_t data[4];
} cvec_t;

static cvec_t add(cvec_t lhs, cvec_t rhs) {
    cvec_t r = {{
        lhs.data[0] + rhs.data[0],
        lhs.data[1] + rhs.data[1],
        lhs.data[2] + rhs.data[2],
        lhs.data[3] + rhs.data[3],
    }};
    return r;
}
static cvec_t sub(cvec_t lhs, cvec_t rhs) {
    cvec_t r = {{
        lhs.data[0] - rhs.data[0],
        lhs.data[1] - rhs.data[1],
        lhs.data[2] - rhs.data[2],
        lhs.data[3] - rhs.data[3],
    }};
    return r;
}
static cvec_t mul(cvec_t lhs, cvec_t rhs) {
    cvec_t r = {{
        lhs.data[0] * rhs.data[0],
        lhs.data[1] * rhs.data[1],
        lhs.data[2] * rhs.data[2],
        lhs.data[3] * rhs.data[3],
    }};
    return r;
}
static cvec_t from_floating(float_t f) {
    cvec_t r = {{f, f, f, f}};
    return r;
}
static cvec_t get_w(floating_t angle) {
    complex floating_t w = cos(angle) + sin(angle) * I;
    cvec_t wv = {{w, w, w, w}};
    return wv;
}

static int bit_reverse(int data, int length) {
    int new_data = 0;
    for (int i = 0; i < length; i++) {
        new_data <<= 1;
        if (data & (1 << i))
            new_data += 1;
    }
    return new_data;
}

typedef struct {
    uint32_t slice;
    uint32_t log_width;
    uint32_t log_height;
    cvec_t *data;
    bool vertical;
} slice_t;

static uint32_t to_index(uint32_t width, uint32_t x, uint32_t y) {
    return width*y + x;
}

static cvec_t sample(slice_t slice, uint32_t t) {
    uint32_t index;
    if (slice.vertical) {
        index = to_index(1<<slice.log_width, slice.slice, t);
    } else {
        index = to_index(1<<slice.log_width, t, slice.slice);
    }
    return slice.data[index];
}

static void write(slice_t slice, uint32_t t, cvec_t data) {
    uint32_t index;
    if (slice.vertical) {
        index = to_index(1<<slice.log_width, slice.slice, t);
    } else {
        index = to_index(1<<slice.log_width, t, slice.slice);
    }
    slice.data[index] = data;
}

static void fft_prologue(int index, slice_t src, slice_t dst, int log_len) {
    int half_len = 1<<(log_len - 1);
    int t = bit_reverse(index, log_len);
    write(dst, index, sample(src, t^half_len));
}

typedef struct {
} iteration_t;
static void fft_iteration(int index, slice_t src, slice_t dst, int iteration, bool inverse) {
    float_t angle = (2*M_PI) / (float_t)(1 << (iteration + 1));
    if (inverse) {
        angle = -angle;
    }
    bool subtract = index & (1 << iteration);

    cvec_t n1 = sample(src, index ^ (1 << iteration));
    cvec_t n2 = sample(src, index);

    int mask = (1 << iteration) - 1;
    int p = index & mask;
    cvec_t w = get_w((p+1)*angle);
    if (index & (1 << iteration)) {
        n2 = mul(n2, w);
    } else {
        n1 = mul(n1, w);
    }

    if (subtract)
        write(dst, index, sub(n1, n2));
    else
        write(dst, index, add(n1, n2));
}

static void fft_epilogue (int index, slice_t src, slice_t dst, int log_len, bool inverse) {
    int half_len = 1<<(log_len - 1);
    cvec_t out = sample(src, (1<<log_len)-index-1);
    if (inverse)
        out = mul(out, from_floating(1.0/((float_t)(1<<log_len))));
    write(dst, index^half_len, out);
}

// *buff_in and *buff_out must be valid memory
// *out_ptr is the pointer to the output buffer
typedef struct {
    cvec_t *restrict buff_in;
    cvec_t *restrict buff_out;
    void **out_ptr;
    slice_t slice_in;
    slice_t slice_out;
    uint32_t log_len;
    bool inverse;
} fft1d_params_t;
static void fft1d(fft1d_params_t *params) {
    cvec_t *buff_in = params->buff_in;
    cvec_t *buff_out = params->buff_out;
    slice_t slice_in = params->slice_in;
    slice_t slice_out = params->slice_out;
    uint32_t log_len = params->log_len;
    bool inverse = params->inverse;
    uint32_t length = 1<<log_len;

    for (uint32_t i = 0; i < length; i++) {
        fft_prologue(i, slice_in, slice_out, log_len);
    }
    {
        void *tmp = buff_in;
        buff_in = buff_out;
        buff_out = tmp;
        slice_in.data = buff_in;
        slice_out.data = buff_out;
    }

    for (uint32_t iter = 0; iter < log_len; iter++) {
        for (uint32_t i = 0; i < length; i++) {
            fft_iteration(i, slice_in, slice_out, iter, inverse);
        }
        void *tmp = buff_in;
        buff_in = buff_out;
        buff_out = tmp;
        slice_in.data = buff_in;
        slice_out.data = buff_out;
    }

    for (uint32_t i = 0; i < length; i++) {
        fft_epilogue(i, slice_in, slice_out, log_len, inverse);
    }
    *(params->out_ptr) = buff_out;
}

typedef struct {
    void (*fn)(void*);
    void *params;
} work_item_t;

typedef struct {
    atomic_uint work_index;
    uint work_item_count;
    work_item_t *work_items;
} work_queue_t;

void *thread_fn(void *param) {
    work_queue_t *p = (work_queue_t*)param;
    while (true) {
        uint32_t index = atomic_fetch_add(&p->work_index, 1);
        if (index >= p->work_item_count)
            break;

        work_item_t *item = &p->work_items[index];
        item->fn(item->params);
    }
    return NULL;
}

enum return_code_t {
    Success = 0,
    Mem = 1,
    Dim = 2,
    Thread = 3,
};

uint8_t fft2d(cvec_t *data, uint32_t log_width, uint32_t log_height, uint32_t threads, bool inverse) {
    uint32_t width = 1<<log_width;
    uint32_t height = 1<<log_height;
    uint32_t data_len = width * height;

    if (!threads) {
        return Thread;
    }
    if ((width == 0) || (height == 0)) {
        return Dim;
    }

    cvec_t *buff = malloc(data_len * sizeof(*data));
    if (!buff) {
        return Mem;
    }
    cvec_t *buff_input = data;
    cvec_t *buff_output = buff;

    pthread_t pid[threads];
    fft1d_params_t params1[height];
    work_item_t work_items1[height];
    work_queue_t queue1 = {
        0, height, work_items1
    };
    fft1d_params_t params2[width];
    work_item_t work_items2[width];
    work_queue_t queue2 = {
        0, width, work_items2
    };

    void *out = NULL;
    for (uint32_t i = 0; i < height; i++) {
        slice_t src = {
            i, log_width, log_height, buff_input, false
        };
        slice_t dst = {
            i, log_width, log_height, buff_output, false
        };
        fft1d_params_t params = {
            buff_input, buff_output, &out, src, dst, log_width, inverse
        };
        params1[i] = params;
        work_item_t work_item = {
            (void(*)(void*))&fft1d, &params1[i]
        };
        work_items1[i] = work_item;
    }
    for (uint32_t i = 0; i < threads; i++) {
        pthread_create(&pid[i], NULL, thread_fn, &queue1);
    }
    for (uint32_t i = 0; i < threads; i++) {
        pthread_join(pid[i], NULL);
    }

    if (!out) { __builtin_unreachable(); }
    buff_input = out;
    buff_output = (out == buff) ? data : buff;

    for (uint32_t i = 0; i < width; i++) {
        slice_t src = {
            i, log_width, log_height, buff_input, true
        };
        slice_t dst = {
            i, log_width, log_height, buff_output, true
        };
        fft1d_params_t params = {
            buff_input, buff_output, &out, src, dst, log_height, inverse
        };
        params2[i] = params;
        work_item_t work_item = {
            (void(*)(void*))&fft1d, &params2[i]
        };
        work_items2[i] = work_item;
    }
    for (uint32_t i = 0; i < threads; i++) {
        pthread_create(&pid[i], NULL, &thread_fn, &queue2);
    }
    for (uint32_t i = 0; i < threads; i++) {
        pthread_join(pid[i], NULL);
    }


    if (out != data) {
        memcpy(data, out, data_len * sizeof(*data));
    }
    free(buff);
    return Success;
}
