import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { HttpResponse } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { IJobOffer } from 'app/entities/job-offer/job-offer.model';
import { JobOfferService } from 'app/entities/job-offer/service/job-offer.service';
import { IRadarHit } from '../radar-hit.model';
import { RadarHitService } from '../service/radar-hit.service';

import { RadarHitFormService } from './radar-hit-form.service';
import { RadarHitUpdate } from './radar-hit-update';

describe('RadarHit Management Update Component', () => {
  let comp: RadarHitUpdate;
  let fixture: ComponentFixture<RadarHitUpdate>;
  let activatedRoute: ActivatedRoute;
  let radarHitFormService: RadarHitFormService;
  let radarHitService: RadarHitService;
  let jobOfferService: JobOfferService;

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

    fixture = TestBed.createComponent(RadarHitUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    radarHitFormService = TestBed.inject(RadarHitFormService);
    radarHitService = TestBed.inject(RadarHitService);
    jobOfferService = TestBed.inject(JobOfferService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should call JobOffer query and add missing value', () => {
      const radarHit: IRadarHit = { id: 20377 };
      const jobOffer: IJobOffer = { id: 9246 };
      radarHit.jobOffer = jobOffer;

      const jobOfferCollection: IJobOffer[] = [{ id: 9246 }];
      vitest.spyOn(jobOfferService, 'query').mockReturnValue(of(new HttpResponse({ body: jobOfferCollection })));
      const additionalJobOffers = [jobOffer];
      const expectedCollection: IJobOffer[] = [...additionalJobOffers, ...jobOfferCollection];
      vitest.spyOn(jobOfferService, 'addJobOfferToCollectionIfMissing').mockReturnValue(expectedCollection);

      activatedRoute.data = of({ radarHit });
      comp.ngOnInit();

      expect(jobOfferService.query).toHaveBeenCalled();
      expect(jobOfferService.addJobOfferToCollectionIfMissing).toHaveBeenCalledWith(
        jobOfferCollection,
        ...additionalJobOffers.map(i => expect.objectContaining(i) as typeof i),
      );
      expect(comp.jobOffersSharedCollection()).toEqual(expectedCollection);
    });

    it('should update editForm', () => {
      const radarHit: IRadarHit = { id: 20377 };
      const jobOffer: IJobOffer = { id: 9246 };
      radarHit.jobOffer = jobOffer;

      activatedRoute.data = of({ radarHit });
      comp.ngOnInit();

      expect(comp.jobOffersSharedCollection()).toContainEqual(jobOffer);
      expect(comp.radarHit).toEqual(radarHit);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IRadarHit>();
      const radarHit = { id: 25582 };
      vitest.spyOn(radarHitFormService, 'getRadarHit').mockReturnValue(radarHit);
      vitest.spyOn(radarHitService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ radarHit });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(radarHit);
      saveSubject.complete();

      // THEN
      expect(radarHitFormService.getRadarHit).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(radarHitService.update).toHaveBeenCalledWith(expect.objectContaining(radarHit));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IRadarHit>();
      const radarHit = { id: 25582 };
      vitest.spyOn(radarHitFormService, 'getRadarHit').mockReturnValue({ id: null });
      vitest.spyOn(radarHitService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ radarHit: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(radarHit);
      saveSubject.complete();

      // THEN
      expect(radarHitFormService.getRadarHit).toHaveBeenCalled();
      expect(radarHitService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IRadarHit>();
      const radarHit = { id: 25582 };
      vitest.spyOn(radarHitService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ radarHit });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(radarHitService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });

  describe('Compare relationships', () => {
    describe('compareJobOffer', () => {
      it('should forward to jobOfferService', () => {
        const entity = { id: 9246 };
        const entity2 = { id: 5985 };
        vitest.spyOn(jobOfferService, 'compareJobOffer');
        comp.compareJobOffer(entity, entity2);
        expect(jobOfferService.compareJobOffer).toHaveBeenCalledWith(entity, entity2);
      });
    });
  });
});
