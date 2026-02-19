#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;

// Input uniform values
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform float scanline;
uniform int fancy;
// Output fragment color
out vec4 finalColor;
float cbrt256 = 6.34960420787;
float div = 6.0;

void main() {
  // Texel color fetching from texture sampler
  // vec4 texelColor = texture(texture0, fragTexCoord);

  // NOTE: Implement here your fragment shader code

  // final color is the color from the texture
  //    times the tint color (colDiffuse)
  //    times the fragment color (interpolated vertex color)
  // finalColor = texelColor * colDiffuse * fragColor;

  vec4 base = texture(texture0, fragTexCoord);
  /*if (fancy == 0) {
    finalColor = base;
    return;
  }*/
  vec4 col = vec4(0.0, 0.0, 0.0, 1.0);
  col.rgb = base.rgb;
  col.a = base.a;
  float count = 480;
  float h = fragTexCoord.y * count;
  float p = count - scanline * count;
  float line_delta = abs(round(h) - h);
  if (line_delta < 0.25) {
    if (line_delta < 0.01) {
      line_delta = 0.01;
    }
    col.rgb += (1. / line_delta * sqrt(line_delta)) * 0.09;
  }
  if (abs(h - p) < 0.1) {
    float delt = abs(h - p);
    delt *= 10;
    if (delt < 0.1) {
      delt = 0.1;
    }
    col.rgb += 0.05;
    // col.rgb *= 1.1;
  }
  finalColor = col;
}
