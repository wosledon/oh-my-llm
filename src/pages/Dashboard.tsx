import { useEffect, useState } from 'react';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Switch from '@mui/material/Switch';

import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import ListSubheader from '@mui/material/ListSubheader';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import { alpha } from '@mui/material/styles';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import CheckIcon from '@mui/icons-material/Check';
import OpenInNewIcon from '@mui/icons-material/OpenInNew';
import DnsIcon from '@mui/icons-material/Dns';
import ViewInArIcon from '@mui/icons-material/ViewInAr';

import { useProxyStore } from '../stores/proxyStore';
import { useProviderStore } from '../stores/providerStore';
import { useModelStore } from '../stores/modelStore';
import { useI18n } from '../i18n';

export default function Dashboard() {
  const { t } = useI18n();
  const { config, running, getStatus, startProxy, stopProxy, updateConfig, fetchConfig } = useProxyStore();
  const { providers, fetchProviders } = useProviderStore();
  const { models, fetchModels } = useModelStore();
  const [copied, setCopied] = useState(false);

  // 本地输入状态，避免直接绑定 store 导致输入被覆盖
  const [portInput, setPortInput] = useState(String(config?.port ?? 11888));
  const [shadowNameInput, setShadowNameInput] = useState(config?.shadow_model_name ?? 'oh-my-llm');

  useEffect(() => {
    getStatus();
    fetchProviders();
    fetchModels();
    fetchConfig();
  }, [getStatus, fetchProviders, fetchModels, fetchConfig]);

  // store config 变化时同步本地状态（但只在外部变化时）
  useEffect(() => {
    if (config) {
      setPortInput(String(config.port));
      setShadowNameInput(config.shadow_model_name);
    }
  }, [config?.port, config?.shadow_model_name]);

  const proxyAddress = config ? `http://127.0.0.1:${config.port}/v1` : '';

  const handleCopy = async () => {
    if (!proxyAddress) return;
    try {
      await navigator.clipboard.writeText(proxyAddress);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // ignore
    }
  };

  const enabledModels = models.filter((m) => m.enabled).length;

  const iconBg = (paletteKey: 'success' | 'error' | 'primary' | 'info' | 'warning' | 'secondary') => ({
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: 36,
    height: 36,
    borderRadius: 1.5,
    bgcolor: (theme: any) => alpha(theme.palette[paletteKey].main, 0.12),
    color: (theme: any) => theme.palette[paletteKey].main,
  });

  const ProtocolSwitch = ({
    checked,
    onChange,
    label,
    color,
  }: {
    checked: boolean;
    onChange: (v: boolean) => void;
    label: string;
    color: 'primary' | 'secondary' | 'warning';
  }) => (
    <Box
      onClick={() => onChange(!checked)}
      sx={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: 1.5,
        px: 2,
        py: 1.25,
        borderRadius: 2,
        border: '1px solid',
        borderColor: checked ? `${color}.main` : 'divider',
        bgcolor: checked ? `${color}.50` : 'background.paper',
        cursor: 'pointer',
        transition: 'all 0.2s',
        '&:hover': { borderColor: checked ? `${color}.dark` : 'action.active' },
      }}
    >
      <Typography variant="body2" sx={{ fontWeight: 600, color: checked ? `${color}.dark` : 'text.primary' }}>
        {label}
      </Typography>
      <Switch
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        color={color}
        sx={{ pointerEvents: 'none' }}
      />
    </Box>
  );

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2.5 }}>
      {/* 第一行：代理状态 + 代理配置（地址+端口+协议+统计） */}
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', md: '1fr 1.2fr' },
          gap: 2.5,
        }}
      >
        {/* 代理状态卡片 */}
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ p: 3 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
              <Box sx={iconBg(running ? 'success' : 'error')}>
                {running ? <StopIcon fontSize="small" /> : <PlayArrowIcon fontSize="small" />}
              </Box>
              <Box sx={{ flex: 1 }}>
                <Typography variant="h6" sx={{ fontWeight: 700, lineHeight: 1.3 }}>
                  {t.dashboard.proxyStatus}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {running ? t.dashboard.running : t.dashboard.stopped}
                </Typography>
              </Box>
              <Chip
                size="small"
                color={running ? 'success' : 'error'}
                label={running ? t.dashboard.running : t.dashboard.stopped}
                sx={{ fontWeight: 600 }}
              />
            </Box>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mb: 2 }}>
              <Typography variant="h3" sx={{ fontWeight: 800, lineHeight: 1 }}>
                {config ? config.port : '--'}
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ pt: 0.5 }}>
                {t.dashboard.port}
              </Typography>
            </Box>
            <Button
              variant={running ? 'outlined' : 'contained'}
              size="small"
              startIcon={running ? <StopIcon /> : <PlayArrowIcon />}
              color={running ? 'error' : 'primary'}
              onClick={async () => {
                if (running) {
                  await stopProxy();
                } else {
                  await startProxy();
                }
                await fetchConfig();
                await getStatus();
              }}
              fullWidth
            >
              {running ? t.dashboard.stop : t.dashboard.start}
            </Button>
          </CardContent>
        </Card>

        {/* 代理配置卡片：地址 + 端口 + 协议 + 统计 */}
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ p: 3 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
              <Box sx={iconBg('primary')}>
                <OpenInNewIcon fontSize="small" />
              </Box>
              <Typography variant="h6" sx={{ fontWeight: 700 }}>
                {t.dashboard.proxyAddress}
              </Typography>
            </Box>

            <Box
              sx={{
                bgcolor: 'action.hover',
                borderRadius: 2,
                p: 1.5,
                display: 'flex',
                alignItems: 'center',
                gap: 1.5,
                mb: 2,
              }}
            >
              <Typography
                variant="body2"
                sx={{ fontFamily: 'monospace', fontWeight: 600, flex: 1, wordBreak: 'break-all' }}
              >
                {proxyAddress || '--'}
              </Typography>
              <Button
                variant="text"
                size="small"
                startIcon={copied ? <CheckIcon fontSize="small" /> : <ContentCopyIcon fontSize="small" />}
                onClick={handleCopy}
                disabled={!proxyAddress}
                sx={{ minWidth: 0, px: 1 }}
              >
                {copied ? t.dashboard.copied : t.dashboard.copyAddress}
              </Button>
            </Box>

            {/* 端口 + 协议 */}
            <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', mb: 2 }}>
              <TextField
                size="small"
                type="number"
                label={t.settings.port}
                value={portInput}
                onChange={(e) => setPortInput(e.target.value)}
                onBlur={() => {
                  if (!config) return;
                  const port = parseInt(portInput) || 11888;
                  if (port !== config.port) {
                    updateConfig({ ...config, port });
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    (e.target as HTMLInputElement).blur();
                  }
                }}
                sx={{ width: 120 }}
              />
              <ProtocolSwitch
                checked={config?.openai_enabled ?? true}
                onChange={(v) => {
                  if (!config) return;
                  updateConfig({ ...config, openai_enabled: v });
                }}
                label={t.settings.openaiEnabled}
                color="primary"
              />
              <ProtocolSwitch
                checked={config?.anthropic_enabled ?? true}
                onChange={(v) => {
                  if (!config) return;
                  updateConfig({ ...config, anthropic_enabled: v });
                }}
                label={t.settings.anthropicEnabled}
                color="secondary"
              />
            </Box>

            {/* 统计条 */}
            <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <DnsIcon fontSize="small" color="action" />
                <Typography variant="body2" color="text.secondary">
                  <strong>{providers.length}</strong> {t.dashboard.providerCount}
                </Typography>
              </Box>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <ViewInArIcon fontSize="small" color="action" />
                <Typography variant="body2" color="text.secondary">
                  <strong>{models.length}</strong> {t.dashboard.modelCount} / <strong>{enabledModels}</strong>{t.dashboard.enabledCount}
                </Typography>
              </Box>
            </Box>
          </CardContent>
        </Card>
      </Box>

      {/* 第二行：影子模型单独卡片 */}
      <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
        <CardContent sx={{ p: 3 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
            <Box sx={iconBg('warning')}>
              <ViewInArIcon fontSize="small" />
            </Box>
            <Typography variant="h6" sx={{ fontWeight: 700 }}>
              {t.dashboard.shadowModel}
            </Typography>
          </Box>

          <Box sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', md: '1fr 1fr' }, gap: 3 }}>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
              <TextField
                size="small"
                label={t.settings.shadowModelName}
                value={shadowNameInput}
                onChange={(e) => setShadowNameInput(e.target.value)}
                onBlur={() => {
                  if (!config) return;
                  if (shadowNameInput !== config.shadow_model_name) {
                    updateConfig({ ...config, shadow_model_name: shadowNameInput });
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    (e.target as HTMLInputElement).blur();
                  }
                }}
                placeholder="oh-my-llm"
                helperText={t.settings.shadowModelDesc}
                fullWidth
              />
            </Box>

            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
              <ProtocolSwitch
                checked={!!config?.shadow_mapping_id}
                onChange={(v) => {
                  if (!config) return;
                  const shadow_mapping_id = v
                    ? (config.shadow_mapping_id || models[0]?.id || '')
                    : null;
                  updateConfig({ ...config, shadow_mapping_id });
                }}
                label={t.dashboard.shadowModelSwitch}
                color="warning"
              />
              {!!config?.shadow_mapping_id && (
                <FormControl fullWidth size="small">
                  <InputLabel>{t.dashboard.shadowModelSelectLabel}</InputLabel>
                  <Select
                    value={config.shadow_mapping_id}
                    label={t.dashboard.shadowModelSelectLabel}
                    onChange={(e) => {
                      if (!config) return;
                      updateConfig({ ...config, shadow_mapping_id: e.target.value });
                    }}
                    MenuProps={{
                      slotProps: {
                        paper: {
                          sx: {
                            mt: 1,
                            borderRadius: 2,
                            boxShadow: (theme: any) => theme.shadows[8],
                            minWidth: 260,
                          },
                        },
                      },
                    }}
                  >
                    {providers
                      .filter((p) => models.some((m) => m.provider_id === p.id))
                      .map((provider, pIdx, filteredProviders) => {
                        const providerModels = models.filter((m) => m.provider_id === provider.id);
                        return [
                          <ListSubheader
                            key={`sub-${provider.id}`}
                            sx={{ fontWeight: 700, bgcolor: 'background.paper' }}
                          >
                            {provider.name}
                          </ListSubheader>,
                          ...providerModels.map((m) => (
                            <MenuItem key={m.id} value={m.id} sx={{ pl: 3 }}>
                              {m.upstream_name}
                            </MenuItem>
                          )),
                          pIdx < filteredProviders.length - 1 && (
                            <Divider key={`div-${provider.id}`} component="li" />
                          ),
                        ];
                      })}
                  </Select>
                </FormControl>
              )}
            </Box>
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
}
