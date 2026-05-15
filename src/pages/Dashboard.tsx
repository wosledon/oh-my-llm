import { useEffect, useState } from 'react';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import ListSubheader from '@mui/material/ListSubheader';
import Divider from '@mui/material/Divider';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import CheckIcon from '@mui/icons-material/Check';
import OpenInNewIcon from '@mui/icons-material/OpenInNew';
import DnsIcon from '@mui/icons-material/Dns';
import ViewInArIcon from '@mui/icons-material/ViewInAr';
import SettingsIcon from '@mui/icons-material/Settings';
import { useProxyStore } from '../stores/proxyStore';
import { useProviderStore } from '../stores/providerStore';
import { useModelStore } from '../stores/modelStore';
import { useI18n } from '../i18n';

export default function Dashboard() {
  const { t } = useI18n();
  const { config, running, getStatus, startProxy, stopProxy, updateConfig } = useProxyStore();
  const { providers, fetchProviders } = useProviderStore();
  const { models, fetchModels } = useModelStore();
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    getStatus();
    fetchProviders();
    fetchModels();
  }, [getStatus, fetchProviders, fetchModels]);

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

  return (
    <Box>
      <Typography variant="h4" sx={{ fontWeight: 600, mb: 3 }}>
        {t.dashboard.title}
      </Typography>

      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
          gap: 3,
        }}
      >
        {/* Proxy Status Card */}
        <Card variant="outlined" sx={{ borderRadius: 2 }}>
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                {t.dashboard.proxyStatus}
              </Typography>
              <Chip
                size="small"
                color={running ? 'success' : 'error'}
                label={running ? t.dashboard.running : t.dashboard.stopped}
              />
            </Box>
            <Typography variant="h3" sx={{ fontWeight: 700, mt: 2 }}>
              {config ? `${config.port}` : '--'}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.port}
            </Typography>
            <Box sx={{ mt: 2 }}>
              <Button
                variant="contained"
                size="small"
                startIcon={running ? <StopIcon /> : <PlayArrowIcon />}
                color={running ? 'inherit' : 'primary'}
                onClick={running ? stopProxy : startProxy}
              >
                {running ? t.dashboard.stop : t.dashboard.start}
              </Button>
            </Box>
          </CardContent>
        </Card>

        {/* Proxy Address Card */}
        <Card variant="outlined" sx={{ borderRadius: 2 }}>
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                {t.dashboard.proxyAddress}
              </Typography>
              <OpenInNewIcon fontSize="small" sx={{ color: 'text.secondary' }} />
            </Box>
            <Typography
              variant="h6"
              sx={{ fontWeight: 600, mt: 2, fontFamily: 'monospace', wordBreak: 'break-all' }}
            >
              {proxyAddress || '--'}
            </Typography>
            <Box sx={{ mt: 2 }}>
              <Button
                variant="outlined"
                size="small"
                startIcon={copied ? <CheckIcon /> : <ContentCopyIcon />}
                onClick={handleCopy}
                disabled={!proxyAddress}
              >
                {copied ? t.dashboard.copied : t.dashboard.copyAddress}
              </Button>
            </Box>
          </CardContent>
        </Card>

        {/* Shadow Model Card */}
        <Card variant="outlined" sx={{ borderRadius: 2 }}>
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                {t.dashboard.shadowModel}
              </Typography>
              <ViewInArIcon fontSize="small" sx={{ color: 'text.secondary' }} />
            </Box>
            <Typography variant="h5" sx={{ fontWeight: 600, mt: 2, fontFamily: 'monospace' }}>
              {config?.shadow_model_name || '--'}
            </Typography>
            <Box sx={{ mt: 2 }}>
              <FormControlLabel
                control={
                  <Switch
                    size="small"
                    checked={!!config?.shadow_mapping_id}
                    onChange={async (e) => {
                      if (!config) return;
                      const shadow_mapping_id = e.target.checked
                        ? (config.shadow_mapping_id || models[0]?.id || '')
                        : null;
                      await updateConfig({ ...config, shadow_mapping_id });
                    }}
                    disabled={models.length === 0}
                  />
                }
                label={
                  <Typography variant="body2">
                    {t.dashboard.shadowModelSwitch}
                  </Typography>
                }
              />
              {!!config?.shadow_mapping_id && (
                <FormControl fullWidth size="small" sx={{ mt: 1.5 }}>
                  <InputLabel>映射模型</InputLabel>
                  <Select
                    value={config.shadow_mapping_id}
                    label="映射模型"
                    onChange={async (e) => {
                      if (!config) return;
                      await updateConfig({ ...config, shadow_mapping_id: e.target.value });
                    }}
                  >
                    {providers
                      .filter((p) => models.some((m) => m.provider_id === p.id))
                      .map((provider, pIdx, filteredProviders) => {
                        const providerModels = models.filter((m) => m.provider_id === provider.id);
                        return [
                          <ListSubheader key={`sub-${provider.id}`}>
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
          </CardContent>
        </Card>

        {/* Providers Card */}
        <Card variant="outlined" sx={{ borderRadius: 2 }}>
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                {t.dashboard.providers}
              </Typography>
              <DnsIcon fontSize="small" sx={{ color: 'text.secondary' }} />
            </Box>
            <Typography variant="h3" sx={{ fontWeight: 700, mt: 2 }}>
              {providers.length}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.total}
            </Typography>
          </CardContent>
        </Card>

        {/* Models Card */}
        <Card variant="outlined" sx={{ borderRadius: 2 }}>
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                {t.dashboard.models}
              </Typography>
              <ViewInArIcon fontSize="small" sx={{ color: 'text.secondary' }} />
            </Box>
            <Typography variant="h3" sx={{ fontWeight: 700, mt: 2 }}>
              {models.length}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {enabledModels} enabled
            </Typography>
          </CardContent>
        </Card>

        {/* Protocols Card */}
        <Card variant="outlined" sx={{ borderRadius: 2 }}>
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                Protocols
              </Typography>
              <SettingsIcon fontSize="small" sx={{ color: 'text.secondary' }} />
            </Box>
            <Box sx={{ mt: 2, display: 'flex', gap: 1, flexWrap: 'wrap' }}>
              <Chip
                size="small"
                color={config?.openai_enabled ? 'success' : 'default'}
                label="OpenAI"
              />
              <Chip
                size="small"
                color={config?.anthropic_enabled ? 'success' : 'default'}
                label="Anthropic"
              />
            </Box>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              {config?.openai_enabled || config?.anthropic_enabled ? 'Active' : 'Inactive'}
            </Typography>
          </CardContent>
        </Card>
      </Box>
    </Box>
  );
}
