import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { IRadarState } from '../radar-state.model';
import { RadarStateService } from '../service/radar-state.service';

import { RadarStateFormService } from './radar-state-form.service';
import { RadarStateUpdate } from './radar-state-update';

describe('RadarState Management Update Component', () => {
  let comp: RadarStateUpdate;
  let fixture: ComponentFixture<RadarStateUpdate>;
  let activatedRoute: ActivatedRoute;
  let radarStateFormService: RadarStateFormService;
  let radarStateService: RadarStateService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(RadarStateUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    radarStateFormService = TestBed.inject(RadarStateFormService);
    radarStateService = TestBed.inject(RadarStateService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should update editForm', () => {
      const radarState: IRadarState = { id: 6100 };

      activatedRoute.data = of({ radarState });
      comp.ngOnInit();

      expect(comp.radarState).toEqual(radarState);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IRadarState>();
      const radarState = { id: 23871 };
      vitest.spyOn(radarStateFormService, 'getRadarState').mockReturnValue(radarState);
      vitest.spyOn(radarStateService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ radarState });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(radarState);
      saveSubject.complete();

      // THEN
      expect(radarStateFormService.getRadarState).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(radarStateService.update).toHaveBeenCalledWith(expect.objectContaining(radarState));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IRadarState>();
      const radarState = { id: 23871 };
      vitest.spyOn(radarStateFormService, 'getRadarState').mockReturnValue({ id: null });
      vitest.spyOn(radarStateService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ radarState: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(radarState);
      saveSubject.complete();

      // THEN
      expect(radarStateFormService.getRadarState).toHaveBeenCalled();
      expect(radarStateService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IRadarState>();
      const radarState = { id: 23871 };
      vitest.spyOn(radarStateService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ radarState });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(radarStateService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });
});
