import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

public class Sample {
    public interface SampleOne extends Library {
        String ParseAstFromFile(String file);

        Pointer ParseAstFromDir(String dir);
        int Add(int a,int b);
    }

    public static void main(String[] args){
        SampleOne sample = Native.load("/Users/pandurang/projects/goastgen/lib-goastgen.dylib", SampleOne.class);

        Pointer res = sample.ParseAstFromDir("/Users/pandurang/projects/golang/helloworld");
        String result = res.getString(0);
        Native.free(Pointer.nativeValue(res));
        System.out.println(result);
    }
}
