using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace LumioClrHostFixture;

/// <summary>
/// MS-00002 Wave 2 clr-host 测试夹具入口。托管方法名与 <see cref="UnmanagedCallersOnlyAttribute"/>
/// 的 EntryPoint 同名（lumio_fixture_entry），使 hostfxr 的 load_assembly_and_get_function_pointer
/// 无论按托管方法名还是按导出名解析都指向同一入口。
/// 行为约定（供 Rust 侧断言）：把 input 字节做 ASCII 小写化后写入 output 并置 bytes_written；
/// output 容量不足时返回 2（映射 BufferTooSmall）并把所需长度写入 bytes_written。
/// </summary>
public static unsafe class FixtureEntry
{
    [UnmanagedCallersOnly(EntryPoint = "lumio_fixture_entry")]
    public static int lumio_fixture_entry(byte* input, int inputLength, byte* output, int outputCapacity, int* bytesWritten)
    {
        if (bytesWritten is null || inputLength < 0)
        {
            return 1;
        }

        if (outputCapacity < inputLength)
        {
            *bytesWritten = inputLength;
            return 2;
        }

        for (var index = 0; index < inputLength; index++)
        {
            var current = input[index];
            output[index] = current is >= (byte)'A' and <= (byte)'Z'
                ? (byte)(current + 32)
                : current;
        }

        *bytesWritten = inputLength;
        return 0;
    }
}
