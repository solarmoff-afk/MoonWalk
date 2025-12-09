// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. UpdateDeveloper, 2025

// На файл распостраняются специальные инструкции препроцессора
// которые должен обработать CLI сборки через замену {{ИМЯ}}
// (Либо пользователь MoonWalk может обработать вручную)
// {{MOON_PACKAGE}} -> Заменяется на реальный пакет приложения
// {{HIDE_SYSTEM_UI}} -> Заменяется на "hideSystemUI();" либо на пустоту
//                      (Это зависит от того, игра у вас или приложение)

package {{MOON_PACKAGE}};

import android.app.NativeActivity;
import android.os.Bundle;
import android.view.View;

public class MainActivity extends NativeActivity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        {{HIDE_SYSTEM_UI}}
    }

    static {
        System.loadLibrary("app");
    }

    /// Функция чтобы скрыть весь системный UI андроид, то есть
    /// верхнюю панель и панель навигации
    private void hideSystemUI() {
        View decorView = getWindow().getDecorView();
        decorView.setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY |
                View.SYSTEM_UI_FLAG_LAYOUT_STABLE |
                View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION |
                View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN |
                View.SYSTEM_UI_FLAG_HIDE_NAVIGATION |
                View.SYSTEM_UI_FLAG_FULLSCREEN
        );
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {
            {{HIDE_SYSTEM_UI}}
        }
    }
}